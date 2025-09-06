package main

import (
	"flag"
	"log"
	"os"
	"path/filepath"
	"time"
)

// Simple troubleshooter to validate host readiness for isolated Arch tests.
// Assumes Docker is already installed and running, and can optionally run a short
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
		archAll     = flag.Bool("arch", false, "Build the Arch Docker image if needed, then run the tests (one-shot)")
		imageTag    = flag.String("image-tag", "oxidizr-arch:latest", "Docker image tag to build/run")
		dockerCtx   = flag.String("docker-context", "test-orch/docker", "Docker build context directory (relative or absolute)")
		rootDirFlag = flag.String("root-dir", "", "Host directory to mount at /workspace (defaults to git root or repo root)")
		noCache     = flag.Bool("no-cache", false, "Build without using cache")
		pullBase    = flag.Bool("pull", false, "Always attempt to pull a newer base image during build")
		keepCtr     = flag.Bool("keep-container", false, "Do not remove container after run (omit --rm)")
		timeout     = flag.Duration("timeout", 30*time.Minute, "Timeout for docker run")
		verbose     = flag.Bool("v", false, "Verbose output")
	)
	flag.Parse()
	log.SetFlags(0)

	// Require root privileges (sudo) for consistent Docker access on systems without docker group configuration.
	if !isRoot() {
		warn("requires root privileges to interact with Docker reliably. Re-run with: sudo go run . [flags]")
		os.Exit(1)
	}

	// Developer-friendly default: with no action flags, perform one-shot build+run
	if !*smokeDocker && !*archBuild && !*archRun && !*archAll {
		*archAll = true
	}

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

	// Orchestrate Docker Arch image build/run if requested, but only if previous checks passed
	if ok && (*archBuild || *archRun || *archAll) {
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

		// If one-shot, we implicitly build first
		doBuild := *archBuild || *archAll
		if doBuild {
			if err := buildArchImage(*imageTag, ctxDir, *noCache, *pullBase, *verbose); err != nil {
				warn("docker build failed: ", err)
				ok = false
			}
		}

		// If running, ensure image exists (auto-build if missing unless user explicitly disabled by not using --arch or --arch-build)
		doRun := *archRun || *archAll
		if doRun {
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

			// Auto-build if the image tag is missing
			if err := runSilent("docker", "image", "inspect", *imageTag); err != nil {
				section("Docker image not found; building")
				if err2 := buildArchImage(*imageTag, ctxDir, *noCache, *pullBase, *verbose); err2 != nil {
					warn("docker build failed: ", err2)
					ok = false
				}
			}
			hostEntrypoint := filepath.Join(rootDir, "test-orch/docker/entrypoint.sh")
			if _, err := os.Stat(hostEntrypoint); err != nil {
				warn("entrypoint not found at ", hostEntrypoint)
				if *verbose {
					log.Println("Ensure --root-dir points to the repository root so test-orch/docker/entrypoint.sh is available.")
				} else {
					log.Println("Hint: set --root-dir to the repo root. Run with -v for details.")
				}
				ok = false
			} else {
				// Run via bash to avoid needing executable bit on the mounted script
				containerCmd := "bash /workspace/test-orch/docker/entrypoint.sh"
				if err := runArchContainer(*imageTag, rootDir, containerCmd, *keepCtr, *timeout, *verbose); err != nil {
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
