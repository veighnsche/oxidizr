package main

import (
	"bytes"
	"context"
	"flag"
	"fmt"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"
)

// Simple troubleshooter to validate host readiness for isolated Arch tests.
// It checks Docker prerequisites and can optionally run a short
// smoke test using a Docker Arch container.
//
// Usage examples:
//   go run .
//   go run . --smoke-arch-docker

func main() {
	var (
		smokeDocker = flag.Bool("smoke-arch-docker", false, "Run a short Arch docker smoke test (pacman + DNS)")
		archBuild   = flag.Bool("arch-build", false, "Build the Arch Docker image used for isolated tests")
		archRun     = flag.Bool("arch-run", false, "Run the Arch Docker container to execute tests via entrypoint.sh")
		imageTag    = flag.String("image-tag", "oxidizr-arch:latest", "Docker image tag to build/run")
		dockerCtx   = flag.String("docker-context", "rust_coreutils_switch/testing/isolated-runner/docker", "Docker build context directory (relative or absolute)")
		rootDirFlag = flag.String("root-dir", "", "Host directory to mount at /workspace (defaults to git root or repo root)")
		noCache     = flag.Bool("no-cache", false, "Build without using cache")
		pullBase    = flag.Bool("pull", false, "Always attempt to pull a newer base image during build")
		keepCtr     = flag.Bool("keep-container", false, "Do not remove container after run (omit --rm)")
		timeout     = flag.Duration("timeout", 30*time.Minute, "Timeout for docker run")
		verbose     = flag.Bool("v", true, "Verbose output")
	)
	flag.Parse()
	log.SetFlags(0)

	ok := true

	// Always perform Docker checks
	if !checkDocker(*verbose) {
		ok = false
	}
	if *smokeDocker {
		if !smokeTestDockerArch(*verbose) {
			ok = false
		}
	}

	// Orchestrate Docker Arch image build/run if requested
	if *archBuild || *archRun {
		// Resolve docker context dir relative to current working dir/repo
		ctxDir := *dockerCtx
		if !filepath.IsAbs(ctxDir) {
			// Try to resolve relative to repo root for convenience
			if root, err := detectRepoRoot(); err == nil {
				ctxDir = filepath.Join(root, ctxDir)
			} else {
				// Fallback to current working directory
				if wd, err2 := os.Getwd(); err2 == nil {
					ctxDir = filepath.Join(wd, *dockerCtx)
				}
			}
		}

		if *archBuild {
			if err := buildArchImage(*imageTag, ctxDir, *noCache, *pullBase, *verbose); err != nil {
				warn("docker build failed: ", err)
				ok = false
			}
		}

		if *archRun {
			// Resolve rootDir to mount
			rootDir := *rootDirFlag
			if rootDir == "" {
				if root, err := detectRepoRoot(); err == nil {
					rootDir = root
				} else {
					// Fall back two directories up from docker context (/workspace expected to contain repo root)
					rootDir = filepath.Clean(filepath.Join(ctxDir, "..", ".."))
				}
			}
			entrypoint := filepath.Join(rootDir, "rust_coreutils_switch/testing/isolated-runner/docker/entrypoint.sh")
			if _, err := os.Stat(entrypoint); err != nil {
				warn("entrypoint not found at ", entrypoint, "; ensure you are pointing root-dir at the repository root")
				ok = false
			} else {
				if err := runArchContainer(*imageTag, rootDir, entrypoint, *keepCtr, *timeout, *verbose); err != nil {
					warn("docker run failed: ", err)
					ok = false
				}
			}
		}
	}

	// LXD checks removed: this troubleshooter is Docker-only

	if ok {
		log.Println("All requested checks passed.")
		os.Exit(0)
	}
	os.Exit(1)
}

func checkDocker(verbose bool) bool {
	section("Docker checks")
	ok := true
	if !have("docker") {
		warn("docker not found on PATH. Install Docker and ensure your user can run it without sudo.")
		return false
	}
	if verbose {
		log.Println(prefixRun(), "docker version --format '{{.Client.Version}}' (client)")
	}
	if err := runSilent("docker", "version"); err != nil {
		warn("docker is installed but not responding. Make sure the Docker daemon is running and your user is in the docker group.")
		ok = false
	}
	return ok
}

func smokeTestDockerArch(verbose bool) bool {
	section("Docker Arch smoke test")
	if !have("docker") {
		warn("docker missing; cannot run smoke test")
		return false
	}
	// Pull a minimal image and run quick commands
	if verbose {
		log.Println(prefixRun(), "docker pull archlinux:base-devel")
	}
	if err := run("docker", "pull", "archlinux:base-devel"); err != nil {
		warn("failed to pull archlinux:base-devel: ", err)
		return false
	}
	cmd := []string{"run", "--rm", "archlinux:base-devel", 
		"bash", "-lc", "set -euo pipefail; pacman -Syy --noconfirm >/dev/null; printf 'nameserver 1.1.1.1\n' >/etc/resolv.conf; ping -c1 -W3 archlinux.org >/dev/null && echo OK"}
	if verbose {
		log.Println(prefixRun(), "docker "+strings.Join(cmd, " "))
	}
	if err := run("docker", cmd...); err != nil {
		warn("Docker Arch smoke test failed. Check network reachability and DNS. Error: ", err)
		return false
	}
	log.Println("Docker Arch smoke test: OK")
	return true
}

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

// --- helpers ---

func have(name string) bool {
	_, err := exec.LookPath(name)
	return err == nil
}

func out(name string, args ...string) string {
	cmd := exec.Command(name, args...)
	var b bytes.Buffer
	cmd.Stdout = &b
	cmd.Stderr = &b
	_ = cmd.Run()
	return strings.TrimSpace(b.String())
}

func run(name string, args ...string) error {
	cmd := exec.Command(name, args...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}

func runSilent(name string, args ...string) error {
	cmd := exec.Command(name, args...)
	cmd.Stdout = nil
	cmd.Stderr = nil
	return cmd.Run()
}

func warn(v ...interface{}) {
	log.Println("WARN:", fmt.Sprint(v...))
}

func section(title string) {
	log.Println()
	log.Println("==>", title)
	time.Sleep(10 * time.Millisecond) // keep logs readable
}

func prefixRun() string { return "RUN>" }
