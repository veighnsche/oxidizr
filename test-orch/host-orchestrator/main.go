package main

import (
	"flag"
	"fmt"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"context"
	"sync"
	"time"

	"github.com/fatih/color"

	"host-orchestrator/dockerutil"
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
		archRun     = flag.Bool("run", false, "Run the Docker container to execute tests via the Go runner")
		archShell   = flag.Bool("shell", false, "Open an interactive shell inside the Docker container")
		distros     = flag.String("distros", "arch,manjaro,cachyos,endeavouros", "Comma-separated list of distributions to test. Defaults to all. E.g., --distros=arch")
		dockerCtx   = flag.String("docker-context", "test-orch", "Docker build context directory (relative or absolute)")
		rootDirFlag = flag.String("root-dir", "", "Host directory to mount at /workspace (defaults to git root or repo root)")
		noCache     = flag.Bool("no-cache", false, "Build without using cache")
		pullBase    = flag.Bool("pull", false, "Always attempt to pull a newer base image during build")
		keepCtr     = flag.Bool("keep-container", false, "Do not remove container after run (omit --rm)")
		timeout     = flag.Duration("timeout", 30*time.Minute, "Timeout for docker run")
		verbose     = flag.Bool("v", false, "Verbose output")
		veryVerbose = flag.Bool("vv", false, "Very verbose (trace) output")
		quiet       = flag.Bool("q", false, "Quiet output (only critical errors and final summary)")
		testFilter  = flag.String("test-filter", "", "Run a single test YAML file instead of all tests")
		testCI      = flag.Bool("test-ci", false, "Run local CI tests with act")
		concurrency = flag.Int("concurrency", 4, "Number of distributions to test in parallel")
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
	// Map to dockerutil.Verb for host-side filtering
	var selected dockerutil.Verb
	switch verbosityLevel {
	case 0:
		selected = dockerutil.V0
	case 1:
		selected = dockerutil.V1
	case 2:
		selected = dockerutil.V2
	default:
		selected = dockerutil.V3
	}
	// Apply selection to helpers so hostLog() uses the same filter
	setSelectedVerb(selected)

	// Require root privileges (sudo) for consistent Docker access on systems without docker group configuration.
	if !isRoot() {
		hostLog(dockerutil.V1, "requires root privileges to interact with Docker reliably. Re-run with: sudo go run . [flags]")
		os.Exit(1)
	}

	// Developer-friendly default: with no action flags, perform build+run using the Go runner
	if *testCI {
		if !have("act") {
			hostLog(dockerutil.V1, "'act' is not installed or not in your PATH.")
			hostLog(dockerutil.V1, "Please install it to run local CI tests: https://github.com/nektos/act#installation")
			os.Exit(1)
		}

		repoRoot, err := detectRepoRoot()
		if err != nil {
			log.Fatalf("Failed to detect repository root: %v", err)
		}

		hostLog(dockerutil.V1, "Running CI 'test-orch' job locally with act...")
		// Specify the runner image to make act non-interactive
		cmd := exec.Command("act", "-j", "test-orch", "-P", "ubuntu-latest=catthehacker/ubuntu:act-latest")
		cmd.Dir = repoRoot
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr

		if err := cmd.Run(); err != nil {
			log.Fatalf("act command failed: %v", err)
		}
		os.Exit(0)
	}

	// Developer-friendly default: with no action flags, perform build+run using the Go runner
	if !*smokeDocker && !*archBuild && !*archRun && !*archShell {
		*archBuild = true
		*archRun = true
		if !quietMode {
			hostLog(dockerutil.V1, "==> No action specified, running default: build + test")
		}
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
	distroMap := map[string]string{
		"arch":        "archlinux:base-devel",
		"manjaro":     "manjarolinux/base",
		"cachyos":     "cachyos/cachyos:latest",
		"endeavouros": "alex5402/endeavouros",
	}

	distroList := strings.Split(*distros, ",")

	if ok && (*archBuild || *archRun || *archShell) {
		var wg sync.WaitGroup
		colorPalette := []*color.Color{
			color.New(color.FgCyan),
			color.New(color.FgGreen),
			color.New(color.FgYellow),
			color.New(color.FgBlue),
			color.New(color.FgMagenta),
			color.New(color.FgRed),
		}

		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()

		errs := make(chan error, len(distroList))
		sem := make(chan struct{}, *concurrency)

		// Handle interactive shell in a single, non-parallel flow, then return.
		if *archShell {
			// Resolve requested distro for shell: default to 'arch' when the flag is at its default
			// multi-value, and enforce a single explicit distro otherwise.
			distrosStr := strings.TrimSpace(*distros)
			defaultSet := "arch,manjaro,cachyos,endeavouros"
			if strings.EqualFold(strings.ReplaceAll(distrosStr, " ", ""), strings.ReplaceAll(defaultSet, " ", "")) {
				distrosStr = "arch"
			}
			if strings.Contains(distrosStr, ",") {
				log.Fatalf("--shell accepts a single distro (got %q). Use --distros=arch (default) or a single value.", *distros)
			}

			d := strings.ToLower(strings.TrimSpace(distrosStr))
			if d == "endeavoros" || d == "endeavouros" {
				d = "endeavouros"
			}
			baseImage, imageOk := distroMap[d]
			if !imageOk {
				warn(fmt.Sprintf("unknown distribution '%s' for --shell", d))
				os.Exit(1)
			}

			// Resolve docker context dir relative to repo root when possible
			ctxDir := *dockerCtx
			if !filepath.IsAbs(ctxDir) {
				if root, err := detectRepoRoot(); err == nil {
					ctxDir = filepath.Join(root, ctxDir)
				} else if wd, err2 := os.Getwd(); err2 == nil {
					ctxDir = filepath.Join(wd, *dockerCtx)
				}
			}

			// Compute content hash for tag
			hash := "latest"
			if h, err := computeBuildHash(ctxDir); err == nil && h != "" {
				hash = h
			}
			tag := fmt.Sprintf("oxidizr-%s:%s", d, hash)

			// Resolve rootDir to mount
			rootDir := *rootDirFlag
			if rootDir == "" {
				if root, err := detectRepoRoot(); err == nil {
					rootDir = root
				} else {
					rootDir = filepath.Clean(filepath.Join(ctxDir, "..", ".."))
				}
			}

			// Ensure image exists; auto-build if missing
			if err := runSilent("docker", "image", "inspect", tag); err != nil {
				shellCol := color.New(color.FgCyan)
				if dockerutil.Allowed(selected, dockerutil.V1) {
					log.Printf("%s Docker image not found; building...", shellCol.Sprint(dockerutil.Prefix(d, dockerutil.V1, "HOST")))
				}
				if err2 := dockerutil.BuildArchImage(tag, ctxDir, baseImage, *noCache, *pullBase, selected, d, shellCol); err2 != nil {
					log.Fatalf("docker build failed for --shell: %v", err2)
				}
			}

			// Launch interactive shell (this pre-runs setup_shell.sh then drops into bash -l)
			if err := dockerutil.RunArchInteractiveShell(tag, rootDir, selected, d); err != nil {
				log.Fatalf("interactive shell failed: %v", err)
			}
			// After shell exits, return without running any parallel tasks
			return
		}

		for i, distroName := range distroList {
			wg.Add(1)
			go func(distro string, col *color.Color) {
				sem <- struct{}{}
				defer func() { <-sem }()
				defer wg.Done()

				// Normalize distro aliases
				d := strings.ToLower(strings.TrimSpace(distro))
				if d == "endeavoros" || d == "endeavouros" {
					d = "endeavouros"
				}

				baseImage, imageOk := distroMap[d]
				if !imageOk {
					errs <- fmt.Errorf("unknown distribution '%s', skipping", distro)
					return
				}
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

				// Compute a content hash of the build inputs to create a unique tag per source revision
				hash := "latest"
				if h, err := computeBuildHash(ctxDir); err == nil && h != "" {
					hash = h
				}
				distroImageTag := fmt.Sprintf("oxidizr-%s:%s", d, hash)
				distroTag := col.Sprintf("[%s]", d)

				// If one-shot, we implicitly build
				doBuild := *archBuild
				if doBuild {
					if dockerutil.Allowed(selected, dockerutil.V1) {
						log.Printf("%s Building test environment (%s)...", col.Sprint(dockerutil.Prefix(d, dockerutil.V1, "HOST")), distroImageTag)
					}
					if err := dockerutil.BuildArchImage(distroImageTag, ctxDir, baseImage, *noCache, *pullBase, selected, d, col); err != nil {
						errs <- fmt.Errorf("%s docker build failed: %w", distroTag, err)
						return
					}
				}

				// If running, ensure image exists (auto-build if missing unless user explicitly disabled by not using --arch or --arch-build)
				doRun := *archRun
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
					if err := runSilent("docker", "image", "inspect", distroImageTag); err != nil {
						if dockerutil.Allowed(selected, dockerutil.V1) {
							log.Printf("%s Docker image not found; building...", col.Sprint(dockerutil.Prefix(d, dockerutil.V1, "HOST")))
						}
						if err2 := dockerutil.BuildArchImage(distroImageTag, ctxDir, baseImage, *noCache, *pullBase, selected, d, col); err2 != nil {
							errs <- fmt.Errorf("%s docker build failed: %w", distroTag, err2)
							return
						}
					}

					// Decide which non-interactive path to run
					var envVars []string
					switch verbosityLevel {
					case 0:
						envVars = append(envVars, "VERBOSE=0")
					case 1:
						envVars = append(envVars, "VERBOSE=1")
					case 2:
						envVars = append(envVars, "VERBOSE=2")
					default:
						envVars = append(envVars, "VERBOSE=3")
						// Propagate verbosity to the Rust binary's logger
						envVars = append(envVars, "RUST_LOG=info")
					}
					if *testFilter != "" {
						envVars = append(envVars, fmt.Sprintf("TEST_FILTER=%s", *testFilter))
					}

					if dockerutil.Allowed(selected, dockerutil.V1) {
						log.Printf("%s Starting tests...", col.Sprint(dockerutil.Prefix(d, dockerutil.V1, "HOST")))
					}
					// Stream and tag per-line with intrinsic levels
					if err := dockerutil.RunArchContainer(ctx, distroImageTag, rootDir, "internal-runner", envVars, *keepCtr, *timeout, selected, d, col); err != nil {
						errs <- fmt.Errorf("%s docker run failed: %w", distroTag, err)
						cancel() // Cancel all other running tests
						return
					}
					if dockerutil.Allowed(selected, dockerutil.V1) {
						log.Printf("%s Tests finished successfully.", col.Sprint(dockerutil.Prefix(d, dockerutil.V1, "HOST")))
					}
				}
			}(distroName, colorPalette[i%len(colorPalette)])
		}

		wg.Wait()

		for err := range errs {
			if err != nil {
				warn(err.Error())
				ok = false
			}
		}
	}

	// LXD checks removed: this troubleshooter is Docker-only

	if ok {
		if !quietMode {
			hostLog(dockerutil.V0, "==> All tests passed successfully.")
		}
		os.Exit(0)
	} else {
		if !quietMode {
			hostLog(dockerutil.V0, "==> Some tests failed.")
		}
	}
	os.Exit(1)
}
