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
	"time"
	"sync"

	"github.com/fatih/color"
)

// classifyLine infers an intrinsic verbosity class and scope from a line.
// Returns (vLevel 0..3, scope "[RUNNER]" or "", content without leading tags).
func classifyLine(line string) (int, string, string) {
	// Detect explicit runner-tagged lines like "[v2][RUNNER] message"
	if strings.HasPrefix(line, "[v") {
		reRunner := regexp.MustCompile(`^\[v([0-3])\]\[RUNNER\]\s+`)
		if m := reRunner.FindStringSubmatch(line); m != nil {
			lvl := int(m[1][0] - '0')
			content := reRunner.ReplaceAllString(line, "")
			return lvl, "[RUNNER]", content
		}
		// Generic [vN] tag (no scope); treat as product/raw intrinsic level
		reGeneric := regexp.MustCompile(`^\[v([0-3])\]\s+`)
		if m := reGeneric.FindStringSubmatch(line); m != nil {
			lvl := int(m[1][0] - '0')
			content := reGeneric.ReplaceAllString(line, "")
			return lvl, "", content
		}
	}
	if strings.HasPrefix(line, "[RUNNER] ") {
		content := strings.TrimPrefix(line, "[RUNNER] ")
		if strings.HasPrefix(content, "RUN> ") {
			return 2, "[RUNNER]", content
		}
		if strings.HasPrefix(content, "CTX> ") {
			return 2, "[RUNNER]", content
		}
		if strings.HasPrefix(content, "TRC> ") {
			return 3, "[RUNNER]", content
		}
		if strings.Contains(content, "❌") {
			return 0, "[RUNNER]", content
		}
		return 1, "[RUNNER]", content
	}
	// Detect Rust env_logger style levels inside product output
	// Map: ERROR->v0, WARN->v1, INFO->v1, DEBUG->v2, TRACE->v3
	switch {
	case strings.Contains(line, " ERROR "):
		return 0, "", line
	case strings.Contains(line, " WARN "):
		return 1, "", line
	case strings.Contains(line, " INFO "):
		return 1, "", line
	case strings.Contains(line, " DEBUG "):
		return 2, "", line
	case strings.Contains(line, " TRACE "):
		return 3, "", line
	}
	// Default for container script/stdout lines
	return 1, "", line
}

func RunArchContainer(parentCtx context.Context, tag, rootDir, command string, envVars []string, keepContainer bool, timeout time.Duration, selected Verb, distro string, col *color.Color) error {
	containerName := fmt.Sprintf("oxidizr-arch-test-%s", strings.ReplaceAll(tag, ":", "-"))
	if Allowed(selected, V2) {
		log.Printf("%s RUN> docker run -v %s:/workspace --name %s %s %s", col.Sprint(Prefix(distro, V2, "HOST")), rootDir, containerName, tag, command)
	}
	ctx, cancel := context.WithTimeout(parentCtx, timeout)
	defer cancel()

	_ = exec.Command("docker", "rm", "-f", containerName).Run()

	args := []string{"run"}
	if !keepContainer {
		args = append(args, "--rm")
	}
	for _, env := range envVars {
		args = append(args, "-e", env)
	}
	// Provide distro identifier to in-container runner for analytics/report naming
	distroKey := strings.TrimPrefix(tag, "oxidizr-")
	if i := strings.Index(distroKey, ":"); i >= 0 {
		distroKey = distroKey[:i]
	}
	args = append(args, "-e", fmt.Sprintf("ANALYTICS_DISTRO=%s", distroKey))
	args = append(args, "-v", fmt.Sprintf("%s:/workspace", rootDir))

	// Add persistent cache mounts to speed up repeated runs
	cacheRoot := filepath.Join(rootDir, ".cache", "test-orch")
	if i := strings.Index(distroKey, ":"); i >= 0 {
		distroKey = distroKey[:i]
	}
	// Namespace caches per-distro to avoid cross-container contention
	cargoReg := filepath.Join(cacheRoot, "cargo", "registry", distroKey)
	cargoGit := filepath.Join(cacheRoot, "cargo", "git", distroKey)
	cargoTarget := filepath.Join(cacheRoot, "cargo-target", distroKey)
	pacmanCache := filepath.Join(cacheRoot, "pacman", distroKey)
	// Make AUR build cache per-distro to avoid concurrent access and cross-distro conflicts
	aurBuild := filepath.Join(cacheRoot, "aur-build", distroKey)
	rustupRoot := filepath.Join(cacheRoot, "rustup", distroKey)
	// Ensure directories exist
	for _, d := range []string{cargoReg, cargoGit, cargoTarget, pacmanCache, aurBuild, rustupRoot} {
		_ = os.MkdirAll(d, 0o755)
	}
	// Bind mounts
	args = append(args, "-v", fmt.Sprintf("%s:%s", cargoReg, "/root/.cargo/registry"))
	args = append(args, "-v", fmt.Sprintf("%s:%s", cargoGit, "/root/.cargo/git"))
	args = append(args, "-v", fmt.Sprintf("%s:%s", cargoTarget, "/workspace/target"))
	args = append(args, "-v", fmt.Sprintf("%s:%s", pacmanCache, "/var/cache/pacman"))
	args = append(args, "-v", fmt.Sprintf("%s:%s", aurBuild, "/home/builder/build"))
	args = append(args, "-v", fmt.Sprintf("%s:%s", rustupRoot, "/root/.rustup"))
	args = append(args, "--workdir", "/workspace")
	args = append(args, "--name", containerName)
	args = append(args, tag)
	if command != "" {
		args = append(args, command)
	}

	cmd := exec.CommandContext(ctx, "docker", args...)

	// Always capture stdout/stderr. In verbose mode we stream to logs; otherwise we
	// keep a bounded ring buffer so failures in quiet mode still surface useful context.
	stdoutPipe, _ := cmd.StdoutPipe()
	stderrPipe, _ := cmd.StderrPipe()

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
			fmt.Println()
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
			fmt.Printf("\r%s [%s] (%d/%d) %s", prefix, bar, x, y, label)
		} else {
			fmt.Printf("\r%s [%s] (%d/%d)", prefix, bar, x, y)
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
					continue
				}
			}
			if Allowed(selected, v) {
				// Host prefixes streamed lines with [distro][vN]; scope omitted to avoid duplication
				log.Printf("%s %s", col.Sprint(Prefix(distro, v, "")), content)
			} else {
				push(&lastStdout, line)
			}
		}
		doneCh <- struct{}{}
	}()
	go func() {
		scanner := bufio.NewScanner(stderrPipe)
		for scanner.Scan() {
			line := scanner.Text()
			lvl, _, content := classifyLine(line)
			v := Verb(lvl)
			if Allowed(selected, v) {
				log.Printf("%s %s", col.Sprint(Prefix(distro, v, "")), content)
			} else {
				push(&lastStderr, line)
			}
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
		return fmt.Errorf("docker run failed (exit code %d). Command: %s\n--- stdout (last %d lines) ---\n%s\n--- stderr (last %d lines) ---\n%s", exitCode, cmdLine, len(lastStdout), stdoutTail, len(lastStderr), stderrTail)
	}
	return nil
}
