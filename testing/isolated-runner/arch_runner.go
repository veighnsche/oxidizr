package main

import (
	"context"
	"fmt"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"
)

// buildArchImage builds the Arch Docker image used for running the isolated tests.
func buildArchImage(tag, contextDir string, noCache, pull bool, verbose bool) error {
	args := []string{"build", "-t", tag}
	if noCache {
		args = append(args, "--no-cache")
	}
	if pull {
		args = append(args, "--pull")
	}
	args = append(args, contextDir)
	if verbose {
		log.Println(prefixRun(), "docker "+strings.Join(args, " "))
	}
	return run("docker", args...)
}

// runArchContainer runs the Arch image with the repo mounted at /workspace and executes entrypoint.sh
func runArchContainer(tag, rootDir, entrypoint string, keepContainer bool, timeout time.Duration, verbose bool) error {
	containerName := "oxidizr-arch-test"
	args := []string{"run"}
	if !keepContainer {
		args = append(args, "--rm")
	}
	args = append(args, "-i", "-v", rootDir+":/workspace", "--name", containerName, tag, entrypoint)
	if verbose {
		log.Println(prefixRun(), "docker "+strings.Join(args, " "))
	}
	// Apply timeout
	ctx, cancel := context.WithTimeout(context.Background(), timeout)
	defer cancel()
	cmd := exec.CommandContext(ctx, "docker", args...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	err := cmd.Run()
	if ctx.Err() == context.DeadlineExceeded {
		return fmt.Errorf("docker run timed out after %s", timeout)
	}
	return err
}

// detectRepoRoot finds the git repository root, or returns an error.
func detectRepoRoot() (string, error) {
	// Prefer `git rev-parse --show-toplevel`
	out := out("git", "rev-parse", "--show-toplevel")
	if out != "" {
		return out, nil
	}
	// Fallback: search upwards for a Cargo.toml or .git directory as heuristic
	wd, err := os.Getwd()
	if err != nil {
		return "", err
	}
	dir := wd
	for i := 0; i < 6; i++ { // don't traverse indefinitely
		if _, err := os.Stat(filepath.Join(dir, ".git")); err == nil {
			return dir, nil
		}
		if _, err := os.Stat(filepath.Join(dir, "Cargo.toml")); err == nil {
			return dir, nil
		}
		parent := filepath.Dir(dir)
		if parent == dir {
			break
		}
		dir = parent
	}
	return "", fmt.Errorf("could not detect repo root")
}
