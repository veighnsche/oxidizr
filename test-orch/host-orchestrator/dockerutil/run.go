package dockerutil

import (
	"bufio"
	"context"
	"fmt"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/fatih/color"
)

func RunArchContainer(parentCtx context.Context, tag, rootDir, command string, envVars []string, keepContainer bool, timeout time.Duration, selected Verb, distro string, col *color.Color) error {
	// Build docker run args and compute derived paths
	opts := RunOptions{
		Tag:           tag,
		RootDir:       rootDir,
		Command:       command,
		EnvVars:       envVars,
		KeepContainer: keepContainer,
		Selected:      selected,
		Distro:        distro,
		Col:           col,
	}
	args, containerName, logsDir := BuildDockerRunArgs(opts)
	if Allowed(selected, V2) {
		log.Printf("%s RUN> docker %s", col.Sprint(Prefix(distro, V2, "HOST")), strings.Join(args, " "))
	}
	ctx, cancel := context.WithTimeout(parentCtx, timeout)
	defer cancel()

	_ = exec.Command("docker", "rm", "-f", containerName).Run()

	cmd := exec.CommandContext(ctx, "docker", args...)

	// Always capture stdout/stderr. In verbose mode we stream to logs; otherwise we
	// keep a bounded ring buffer so failures in quiet mode still surface useful context.
	stdoutPipe, _ := cmd.StdoutPipe()
	stderrPipe, _ := cmd.StderrPipe()

	// Prepare timestamped log files; stderr is not printed to console, stdout is printed per verbosity.
	ts := time.Now().UTC().Format("20060102-150405Z")
	stderrLogPath := filepath.Join(logsDir, fmt.Sprintf("%s-stderr-%s.log", containerName, ts))
	stderrLogFile, err := os.Create(stderrLogPath)
	if err != nil {
		return fmt.Errorf("failed to create stderr log file: %w", err)
	}
	defer stderrLogFile.Close()
	stdoutLogPath := filepath.Join(logsDir, fmt.Sprintf("%s-stdout-%s.log", containerName, ts))
	stdoutLogFile, err := os.Create(stdoutLogPath)
	if err != nil {
		return fmt.Errorf("failed to create stdout log file: %w", err)
	}
	defer stdoutLogFile.Close()
	if Allowed(selected, V2) {
		log.Printf("%s CTX> container stdout -> %s", col.Sprint(Prefix(distro, V2, "HOST")), stdoutLogPath)
		log.Printf("%s CTX> container stderr -> %s", col.Sprint(Prefix(distro, V2, "HOST")), stderrLogPath)
	}

	const maxLines = 200
	lastStdout := make([]string, 0, maxLines)
	lastStderr := make([]string, 0, maxLines)
	push := func(buf *[]string, line string) {
		if len(*buf) < maxLines {
			*buf = append(*buf, line)
			return
		}
		copy((*buf)[0:], (*buf)[1:])
		(*buf)[maxLines-1] = line
	}

	doneCh := make(chan struct{}, 2)
	// Progress-bar protocol (Option A): lines like "PB> x/y [label]" from the runner.
	// We only render the bar in v1 to avoid interference with -v/-vv.
	showPB := selected == V1
	pbRe := regexp.MustCompile(`^PB>\s*(\d+)\s*/\s*(\d+)(?:\s+(.*))?$`)
	var pbMu sync.Mutex
	progressShown := false
	finishPB := func() {
		pbMu.Lock()
		if progressShown {
			// Print newline on stderr to align stream with log.Printf output
			fmt.Fprintln(os.Stderr)
			progressShown = false
		}
		pbMu.Unlock()
	}
	updatePB := func(x, y int, label string) {
		pbMu.Lock()
		// Build a compact bar
		width := 28
		if y <= 0 { y = 1 }
		if x < 0 { x = 0 }
		if x > y { x = y }
		filled := int(float64(width) * float64(x) / float64(y))
		if filled > width { filled = width }
		bar := strings.Repeat("=", filled) + strings.Repeat(" ", width-filled)
		prefix := col.Sprint(Prefix(distro, V1, ""))
		if label != "" {
			fmt.Fprintf(os.Stderr, "\r%s [%s] (%d/%d) %s\x1b[K", prefix, bar, x, y, label)
		} else {
			fmt.Fprintf(os.Stderr, "\r%s [%s] (%d/%d)\x1b[K", prefix, bar, x, y)
		}
		progressShown = true
		pbMu.Unlock()
	}
	go func() {
		scanner := bufio.NewScanner(stdoutPipe)
		for scanner.Scan() {
			line := scanner.Text()
			lvl, _, content := classifyLine(line)
			v := Verb(lvl)
			if showPB && strings.HasPrefix(content, "PB> ") {
				m := pbRe.FindStringSubmatch(content)
				if m != nil {
					x, _ := strconv.Atoi(m[1])
					y, _ := strconv.Atoi(m[2])
					label := m[3]
					updatePB(x, y, label)
					if y > 0 && x >= y { // when complete, finish the line so next logs start on a fresh line
						finishPB()
					}
					// Still persist the frame to stdout log for postmortem
					fmt.Fprintln(stdoutLogFile, content)
					continue
				}
			}
			if Allowed(selected, v) {
				// Host prefixes streamed lines with [distro][vN]; scope omitted to avoid duplication
				log.Printf("%s %s", col.Sprint(Prefix(distro, v, "")), content)
			} else {
				push(&lastStdout, line)
			}
			// Always write container stdout to file
			fmt.Fprintln(stdoutLogFile, content)
		}
		doneCh <- struct{}{}
	}()
	go func() {
		scanner := bufio.NewScanner(stderrPipe)
		for scanner.Scan() {
			line := scanner.Text()
			_, _, content := classifyLine(line)
			// Always capture tail and write to file, do not print to console
			push(&lastStderr, content)
			fmt.Fprintln(stderrLogFile, content)
		}
		doneCh <- struct{}{}
	}()

	if err := cmd.Start(); err != nil {
		return fmt.Errorf("docker run start failed: %w", err)
	}
	runErr := cmd.Wait()
	<-doneCh
	<-doneCh

	// Ensure any in-progress bar is finalized
	finishPB()
	if runErr != nil {
		exitCode := -1
		if ee, ok := runErr.(*exec.ExitError); ok {
			exitCode = ee.ExitCode()
		}
		cmdLine := "docker " + strings.Join(args, " ")
		stdoutTail := strings.Join(lastStdout, "\n")
		stderrTail := strings.Join(lastStderr, "\n")
		return fmt.Errorf("docker run failed (exit code %d). Command: %s\n--- stdout (last %d lines) [full at %s] ---\n%s\n--- stderr (last %d lines) [full at %s] ---\n%s", exitCode, cmdLine, len(lastStdout), stdoutLogPath, stdoutTail, len(lastStderr), stderrLogPath, stderrTail)
	}
	return nil
}
