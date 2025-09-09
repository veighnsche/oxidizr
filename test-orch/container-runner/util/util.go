package util

import (
	"bufio"
	"fmt"
	"log"
	"os"
	"os/exec"
	"strings"

	"container-runner/analytics"
)

// RunCmd executes a command and streams its output to stdout/stderr.
func RunCmd(name string, args ...string) error {
    // Echo command for visibility; host will classify '[RUNNER] RUN>' as v2.
    log.Printf("RUN> %s %s", name, strings.Join(args, " "))
	cmd := exec.Command(name, args...)

	// Environment defaults for better diagnostics
	env := os.Environ()
	env = setOrReplaceEnv(env, "RUST_BACKTRACE", "1")
	// If invoking oxidizr-arch directly, ensure we get logs unless user already set RUST_LOG
	if name == "oxidizr-arch" {
		hasRL := false
		for _, e := range env {
			if strings.HasPrefix(e, "RUST_LOG=") {
				hasRL = true
				break
			}
		}
		if !hasRL {
			env = setOrReplaceEnv(env, "RUST_LOG", "info")
		}
	}
	cmd.Env = env

	stdout, _ := cmd.StdoutPipe()
	stderr, _ := cmd.StderrPipe()
	if err := cmd.Start(); err != nil {
		return fmt.Errorf("failed to start command %q: %w", name, err)
	}
	doneCh := make(chan struct{}, 2)
	go func() {
		scanner := bufio.NewScanner(stdout)
		for scanner.Scan() {
			line := scanner.Text()
			analytics.ProcessLine(line)
			// Host prefixes container stream with [DISTRO]; print plain lines here
			fmt.Fprintln(os.Stdout, line)
		}
		doneCh <- struct{}{}
	}()
	go func() {
		scanner := bufio.NewScanner(stderr)
		for scanner.Scan() {
			line := scanner.Text()
			analytics.ProcessLine(line)
			// Host prefixes container stream with [DISTRO]; print plain lines here
			fmt.Fprintln(os.Stderr, line)
		}
		doneCh <- struct{}{}
	}()
	// Wait for pipes to drain and command to exit
	<-doneCh
	<-doneCh
	if err := cmd.Wait(); err != nil {
		exitCode := -1
		if ee, ok := err.(*exec.ExitError); ok {
			exitCode = ee.ExitCode()
		}
		return fmt.Errorf("command failed (exit %d): %s %v: %w", exitCode, name, args, err)
	}
	return nil
}

// RunCmdQuiet executes a command but does not stream its output.
func RunCmdQuiet(name string, args ...string) error {
	cmd := exec.Command(name, args...)
	// Environment defaults similar to RunCmd
	env := os.Environ()
	env = setOrReplaceEnv(env, "RUST_BACKTRACE", "1")
	if name == "oxidizr-arch" {
		hasRL := false
		for _, e := range env {
			if strings.HasPrefix(e, "RUST_LOG=") {
				hasRL = true
				break
			}
		}
		if !hasRL {
			env = setOrReplaceEnv(env, "RUST_LOG", "info")
		}
	}
	cmd.Env = env

	stdout, _ := cmd.StdoutPipe()
	stderr, _ := cmd.StderrPipe()
	if err := cmd.Start(); err != nil {
		return fmt.Errorf("failed to start command %q: %w", name, err)
	}
	doneCh := make(chan struct{}, 2)
	go func() {
		scanner := bufio.NewScanner(stdout)
		for scanner.Scan() {
			analytics.ProcessLine(scanner.Text())
		}
		doneCh <- struct{}{}
	}()
	go func() {
		scanner := bufio.NewScanner(stderr)
		for scanner.Scan() {
			analytics.ProcessLine(scanner.Text())
		}
		doneCh <- struct{}{}
	}()
	<-doneCh
	<-doneCh
	if err := cmd.Wait(); err != nil {
		exitCode := -1
		if ee, ok := err.(*exec.ExitError); ok {
			exitCode = ee.ExitCode()
		}
		return fmt.Errorf("command failed (exit %d): %s %v: %w", exitCode, name, args, err)
	}
	return nil
}

// Has reports whether a command exists in PATH.
func Has(name string) bool {
	_, err := exec.LookPath(name)
	return err == nil
}

// setOrReplaceEnv sets key=value in the env slice, replacing an existing entry
// if present, or appending if not present. Returns a new slice.
func setOrReplaceEnv(env []string, key, value string) []string {
	prefix := key + "="
	for i, e := range env {
		if len(e) >= len(prefix) && e[:len(prefix)] == prefix {
			// Replace in place
			env[i] = prefix + value
			return env
		}
	}
	// Not found; append
	return append(env, prefix+value)
}
