package dockerutil

import (
	"bufio"
	"context"
	"fmt"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"

	"github.com/fatih/color"
)

func RunArchContainer(parentCtx context.Context, tag, rootDir, command string, envVars []string, keepContainer bool, timeout time.Duration, verbose bool, prefix string, col *color.Color) error {
	containerName := fmt.Sprintf("oxidizr-arch-test-%s", strings.ReplaceAll(tag, ":", "-"))
	if verbose {
		log.Println("RUN>", "docker run", "-v", rootDir+":/workspace", "--name", containerName, tag, command)
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
	go func() {
		scanner := bufio.NewScanner(stdoutPipe)
		for scanner.Scan() {
			line := scanner.Text()
			if verbose {
				log.Printf("%s %s", col.Sprint(prefix), line)
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
			if verbose {
				log.Printf("%s %s", col.Sprint(prefix), line)
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

	if runErr != nil {
		exitCode := -1
		if ee, ok := runErr.(*exec.ExitError); ok {
			exitCode = ee.ExitCode()
		}
		cmdLine := "docker " + strings.Join(args, " ")
		if verbose {
			return fmt.Errorf("docker run failed (exit code %d): %s: %w", exitCode, cmdLine, runErr)
		}
		stdoutTail := strings.Join(lastStdout, "\n")
		stderrTail := strings.Join(lastStderr, "\n")
		return fmt.Errorf("docker run failed (exit code %d). Command: %s\n--- stdout (last %d lines) ---\n%s\n--- stderr (last %d lines) ---\n%s", exitCode, cmdLine, len(lastStdout), stdoutTail, len(lastStderr), stderrTail)
	}
	return nil
}
