package main

import (
	"flag"
	"fmt"
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
//   go run . --arch-shell
//   go run . --arch-test-sudo
//   go run . --arch-shell-test-sudo

func main() {
	var (
		smokeDocker = flag.Bool("smoke-arch-docker", false, "Run a short Arch docker smoke test (pacman + DNS)")
		archBuild   = flag.Bool("arch-build", false, "Build the Arch Docker image used for isolated tests")
		archRun     = flag.Bool("arch-run", false, "Run the Arch Docker container to execute tests via entrypoint.sh")
		archAll     = flag.Bool("arch", false, "Build the Arch Docker image if needed, then run the tests (one-shot)")
		archShell   = flag.Bool("arch-shell", false, "Open an interactive shell inside the Arch Docker container with the repo mounted at /workspace")
		archTestSudo = flag.Bool("arch-test-sudo", false, "Run automated sudo/sudo-rs enable/disable assertions inside the Arch container")
		archShellTestSudo = flag.Bool("arch-shell-test-sudo", false, "Run automated sudo/sudo-rs assertions, then drop into an interactive shell")
		imageTag    = flag.String("image-tag", "oxidizr-arch:latest", "Docker image tag to build/run")
		dockerCtx   = flag.String("docker-context", "test-orch/docker", "Docker build context directory (relative or absolute)")
		rootDirFlag = flag.String("root-dir", "", "Host directory to mount at /workspace (defaults to git root or repo root)")
		noCache     = flag.Bool("no-cache", false, "Build without using cache")
		pullBase    = flag.Bool("pull", false, "Always attempt to pull a newer base image during build")
		keepCtr     = flag.Bool("keep-container", false, "Do not remove container after run (omit --rm)")
		timeout     = flag.Duration("timeout", 30*time.Minute, "Timeout for docker run")
		verbose     = flag.Bool("v", false, "Verbose output")
		veryVerbose = flag.Bool("vv", false, "Very verbose (trace) output")
		quiet       = flag.Bool("q", false, "Quiet output (only critical errors and final summary)")
	)
	flag.Parse()
	log.SetFlags(0)

	// Configure verbosity levels
	var verbosityLevel int
	if *quiet {
		verbosityLevel = 0
	} else if *veryVerbose {
		verbosityLevel = 3
	} else if *verbose {
		verbosityLevel = 2
	} else {
		verbosityLevel = 1
	}
	setQuiet(*quiet)
	setVerbosity(verbosityLevel)

	// Require root privileges (sudo) for consistent Docker access on systems without docker group configuration.
	if !isRoot() {
		warn("requires root privileges to interact with Docker reliably. Re-run with: sudo go run . [flags]")
		os.Exit(1)
	}

	// Developer-friendly default: with no action flags, perform one-shot build+run
	if !*smokeDocker && !*archBuild && !*archRun && !*archAll && !*archShell && !*archTestSudo && !*archShellTestSudo {
		*archAll = true
	}

	ok := true

	// Always perform Docker checks
	if !checkDocker(verbosityLevel >= 2) {
		ok = false
	}
	if *smokeDocker {
		if !smokeTestDockerArch(verbosityLevel >= 2) {
			ok = false
		}
	}

	// Orchestrate Docker Arch image build/run/shell if requested, but only if previous checks passed
	if ok && (*archBuild || *archRun || *archAll || *archShell || *archTestSudo || *archShellTestSudo) {
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
			if err := buildArchImage(*imageTag, ctxDir, *noCache, *pullBase, verbosityLevel >= 2); err != nil {
				warn("docker build failed: ", err)
				ok = false
			}
		}

		// If running, ensure image exists (auto-build if missing unless user explicitly disabled by not using --arch or --arch-build)
		doRun := *archRun || *archAll || *archTestSudo
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
				if err2 := buildArchImage(*imageTag, ctxDir, *noCache, *pullBase, verbosityLevel >= 2); err2 != nil {
					warn("docker build failed: ", err2)
					ok = false
				}
			}
			// Decide which non-interactive path to run
			if ok {
				var hostScript string
				var entryCmd string
				// Propagate verbosity into container for entrypoint.sh to honor
				envPrefix := "env VERBOSE="
				switch verbosityLevel {
				case 0:
					envPrefix += "0"
				case 1:
					envPrefix += "1"
				case 2:
					envPrefix += "2"
				default:
					envPrefix += "3"
				}
				if *archTestSudo {
					hostScript = filepath.Join(rootDir, "test-orch/docker/run_sudo_tests.sh")
					entryCmd = envPrefix + " bash /workspace/test-orch/docker/run_sudo_tests.sh"
				} else {
					hostScript = filepath.Join(rootDir, "test-orch/docker/entrypoint.sh")
					entryCmd = envPrefix + " bash /workspace/test-orch/docker/entrypoint.sh"
				}
				if _, err := os.Stat(hostScript); err != nil {
					warn("required script not found at ", hostScript)
					if verbosityLevel >= 2 {
						log.Println("Ensure --root-dir points to the repository root so the script exists.")
					} else {
						log.Println("Hint: set --root-dir to the repo root. Run with -v for details.")
					}
					ok = false
				} else {
					if err := runArchContainer(*imageTag, rootDir, entryCmd, *keepCtr, *timeout, verbosityLevel >= 2); err != nil {
						warn("docker run failed: ", err)
						ok = false
					}
				}
			}
		}

		// If interactive shell is requested
		if ok && (*archShell || *archShellTestSudo) {
			// Resolve rootDir to mount
			rootDir := *rootDirFlag
			if rootDir == "" {
				if root, err := detectRepoRoot(); err == nil {
					rootDir = root
				} else {
					rootDir = filepath.Clean(filepath.Join(ctxDir, "..", ".."))
				}
			}
			// Auto-build if the image tag is missing
			if err := runSilent("docker", "image", "inspect", *imageTag); err != nil {
				section("Docker image not found; building")
				if err2 := buildArchImage(*imageTag, ctxDir, *noCache, *pullBase, verbosityLevel >= 2); err2 != nil {
					warn("docker build failed: ", err2)
					ok = false
				}
			}
			if ok {
				if *archShellTestSudo {
					// Run tests, then drop into a login shell in the same container
					level := 1
					if verbosityLevel == 0 {
						level = 0
					} else if verbosityLevel >= 2 {
						level = 2
					}
					cmd := fmt.Sprintf("bash -lc 'env VERBOSE=%d bash /workspace/test-orch/docker/run_sudo_tests.sh && exec bash -l'", level)
					if err := runArchInteractiveCommand(*imageTag, rootDir, cmd, verbosityLevel >= 2); err != nil {
						warn("interactive shell with tests failed: ", err)
						ok = false
					}
				} else {
					if err := runArchInteractiveShell(*imageTag, rootDir, verbosityLevel >= 2); err != nil {
						warn("interactive shell failed: ", err)
						ok = false
					}
				}
			}
		}
	}

	// LXD checks removed: this troubleshooter is Docker-only

	if ok {
		if !quietMode {
			log.Println("All requested checks passed.")
		}
		os.Exit(0)
	}
	os.Exit(1)
}
