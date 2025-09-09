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

// RunMeta captures metadata about a docker run invocation for summary reporting.
type RunMeta struct {
	Distro         string
	Tag            string
	ContainerName  string
	ContainerID    string
	StartedAt      time.Time
	FinishedAt     time.Time
	ExitCode       int
	StdoutLogPath  string
	StderrLogPath  string
}

func RunArchContainer(parentCtx context.Context, tag, rootDir, command string, envVars []string, keepContainer bool, timeout time.Duration, selected Verb, distro string, col *color.Color, runID string) (RunMeta, error) {
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
		RunID:         runID,
	}
	args, containerName, logsDir, cidFile := BuildDockerRunArgs(opts)
	meta := RunMeta{Distro: distro, Tag: tag, ContainerName: containerName}
	// Host JSONL logger for lifecycle events
	hostLogger := NewHostJSONLLogger(logsDir, opts.RunID, opts.Distro)
	if Allowed(selected, V1) {
		hostLogger.Event("info", "run", "container_start", "starting docker run", nil, nil)
	}
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

	// Prepare temp log files; we'll delete them on success or rename to timestamped paths on error.
	stdoutTmpFile, err := os.CreateTemp(logsDir, fmt.Sprintf("%s-stdout-*.log", containerName))
	if err != nil {
		return meta, fmt.Errorf("failed to create temp stdout log file: %w", err)
	}
	defer stdoutTmpFile.Close()
	stderrTmpFile, err := os.CreateTemp(logsDir, fmt.Sprintf("%s-stderr-*.log", containerName))
	if err != nil {
		return meta, fmt.Errorf("failed to create temp stderr log file: %w", err)
	}
	defer stderrTmpFile.Close()

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
					// Still persist the frame to stdout temp log for postmortem
					fmt.Fprintln(stdoutTmpFile, content)
					continue
				}
			}
			if Allowed(selected, v) {
				// Host prefixes streamed lines with [distro][vN]; scope omitted to avoid duplication
				log.Printf("%s %s", col.Sprint(Prefix(distro, v, "")), content)
			} else {
				push(&lastStdout, line)
			}
			// Always write container stdout to temp file
			fmt.Fprintln(stdoutTmpFile, content)
		}
		doneCh <- struct{}{}
	}()
	go func() {
		scanner := bufio.NewScanner(stderrPipe)
		for scanner.Scan() {
			line := scanner.Text()
			_, _, content := classifyLine(line)
			// Always capture tail and write to temp file, do not print to console
			push(&lastStderr, content)
			fmt.Fprintln(stderrTmpFile, content)
			// At very-verbose, live-stream stderr as well (do not attempt to reclassify severity)
			if Allowed(selected, V3) {
				log.Printf("%s %s", col.Sprint(Prefix(distro, V3, "")), content)
			}
		}
		doneCh <- struct{}{}
	}()

	meta.StartedAt = time.Now().UTC()
	if err := cmd.Start(); err != nil {
		// Finalize temp logs into stable paths so callers can inspect
		ts := time.Now().UTC().Format("20060102-150405Z")
		finalStdout := filepath.Join(logsDir, fmt.Sprintf("%s-stdout-%s.log", containerName, ts))
		finalStderr := filepath.Join(logsDir, fmt.Sprintf("%s-stderr-%s.log", containerName, ts))
		stdoutTmpFile.Close()
		stderrTmpFile.Close()
		_ = os.Rename(stdoutTmpFile.Name(), finalStdout)
		_ = os.Rename(stderrTmpFile.Name(), finalStderr)
		_ = os.Chmod(finalStdout, 0o644)
		_ = os.Chmod(finalStderr, 0o644)
		meta.FinishedAt = time.Now().UTC()
		meta.StdoutLogPath = finalStdout
		meta.StderrLogPath = finalStderr
		meta.ExitCode = -1
		// Emit exit event to host JSONL
		if Allowed(selected, V1) {
			dur := meta.FinishedAt.Sub(meta.StartedAt).Milliseconds()
			rc := meta.ExitCode
			hostLogger.Event("error", "run", "container_exit", "docker run failed to start", &rc, &dur)
		}
		return meta, fmt.Errorf("docker run start failed: %w", err)
	}
	runErr := cmd.Wait()
	<-doneCh
	<-doneCh

	// Ensure any in-progress bar is finalized
	finishPB()
	meta.FinishedAt = time.Now().UTC()
	// Determine exit code
	exitCode := 0
	if runErr != nil {
		if ee, ok := runErr.(*exec.ExitError); ok {
			exitCode = ee.ExitCode()
		} else {
			exitCode = -1
		}
	}
	meta.ExitCode = exitCode

	// Persist logs with a shared timestamp for both success and failure
	ts := meta.FinishedAt.Format("20060102-150405Z")
	finalStdout := filepath.Join(logsDir, fmt.Sprintf("%s-stdout-%s.log", containerName, ts))
	finalStderr := filepath.Join(logsDir, fmt.Sprintf("%s-stderr-%s.log", containerName, ts))
	stdoutTmpFile.Close()
	stderrTmpFile.Close()
	_ = os.Rename(stdoutTmpFile.Name(), finalStdout)
	_ = os.Rename(stderrTmpFile.Name(), finalStderr)
	_ = os.Chmod(finalStdout, 0o644)
	_ = os.Chmod(finalStderr, 0o644)
	meta.StdoutLogPath = finalStdout
	meta.StderrLogPath = finalStderr

	// Attempt to read container ID from cidfile (may not exist on early start failures)
	if b, err := os.ReadFile(cidFile); err == nil {
		meta.ContainerID = strings.TrimSpace(string(b))
	}
	if meta.ContainerID != "" {
		hostLogger.SetContainerID(meta.ContainerID)
		if Allowed(selected, V1) {
			hostLogger.Event("info", "run", "container_ready", "cid acquired", nil, nil)
		}
	}

	// Mirror artifacts from container to host before any potential removal
	// Destination: <rootDir>/.artifacts/<runID>/<distro>_<containerID>/
	if meta.ContainerID != "" {
		artDir := filepath.Join(rootDir, ".artifacts", runID, fmt.Sprintf("%s_%s", distro, meta.ContainerID))
		_ = os.MkdirAll(artDir, 0o755)
		// docker cp <cid>:/workspace/.proof/. <dest>
		_ = exec.Command("docker", "cp", fmt.Sprintf("%s:%s", meta.ContainerID, "/workspace/.proof/."), artDir).Run()
		// Also copy host.jsonl into the artifact directory
		hostJSON := filepath.Join(logsDir, "host.jsonl")
		if b, err := os.ReadFile(hostJSON); err == nil {
			_ = os.WriteFile(filepath.Join(artDir, "host.jsonl"), b, 0o644)
		}
		hostLogger.Event("info", "artifact", "artifact_mirror", artDir, nil, nil)
	}

	// Best-effort cleanup if container should not be kept
	if !opts.KeepContainer {
		id := meta.ContainerID
		if id == "" {
			id = containerName
		}
		// Try a graceful stop first, then force remove
		_ = exec.Command("docker", "stop", "--time", "10", id).Run()
		_ = exec.Command("docker", "rm", "-f", id).Run()
	}
	// Emit stderr tail at very-verbose for easier triage
	if Allowed(selected, V3) {
		for _, line := range lastStderr {
			// Emit as debug-level tail entries
			hostLogger.Event("debug", "run", "stderr_tail", line, nil, nil)
		}
	}

	if runErr != nil {
		cmdLine := "docker " + strings.Join(args, " ")
		// Emit exit event to host JSONL
		if Allowed(selected, V1) {
			dur := meta.FinishedAt.Sub(meta.StartedAt).Milliseconds()
			rc := meta.ExitCode
			hostLogger.Event("error", "run", "container_exit", cmdLine, &rc, &dur)
		}
		return meta, fmt.Errorf("docker run failed (exit code %d). Command: %s\nLogs saved to:\n  stdout: %s\n  stderr: %s", exitCode, cmdLine, finalStdout, finalStderr)
	}
	// Emit successful exit
	if Allowed(selected, V1) {
		dur := meta.FinishedAt.Sub(meta.StartedAt).Milliseconds()
		rc := meta.ExitCode
		hostLogger.Event("info", "run", "container_exit", "success", &rc, &dur)
	}
	return meta, nil
}
