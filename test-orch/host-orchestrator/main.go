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

	// Require root privileges (sudo) for consistent Docker access on systems without docker group configuration.
	if !isRoot() {
		warn("requires root privileges to interact with Docker reliably. Re-run with: sudo go run . [flags]")
		os.Exit(1)
	}

	// Developer-friendly default: with no action flags, perform build+run using the Go runner
	if *testCI {
		if !have("act") {
			log.Println("'act' is not installed or not in your PATH.")
			log.Println("Please install it to run local CI tests: https://github.com/nektos/act#installation")
			os.Exit(1)
		}

		repoRoot, err := detectRepoRoot()
		if err != nil {
			log.Fatalf("Failed to detect repository root: %v", err)
		}

		log.Println("Running CI 'test-orch' job locally with act...")
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
			log.Println("==> No action specified, running default: build + test")
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
				prefix := col.Sprintf("[%s]", d)

				log.Printf("%s Processing...", prefix)

				// If one-shot, we implicitly build
				doBuild := *archBuild
				if doBuild {
					log.Printf("%s Building test environment (%s)...", prefix, distroImageTag)
					if err := dockerutil.BuildArchImage(distroImageTag, ctxDir, baseImage, *noCache, *pullBase, verbosityLevel >= 2, prefix, col); err != nil {
						errs <- fmt.Errorf("%s docker build failed: %w", prefix, err)
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
						log.Printf("%s Docker image not found; building...", prefix)
						if err2 := dockerutil.BuildArchImage(distroImageTag, ctxDir, baseImage, *noCache, *pullBase, verbosityLevel >= 2, prefix, col); err2 != nil {
							errs <- fmt.Errorf("%s docker build failed: %w", prefix, err2)
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

					log.Printf("%s Starting tests...", prefix)
					if err := dockerutil.RunArchContainer(ctx, distroImageTag, rootDir, "internal-runner", envVars, *keepCtr, *timeout, verbosityLevel >= 1, prefix, col); err != nil {
						errs <- fmt.Errorf("%s docker run failed: %w", prefix, err)
						cancel() // Cancel all other running tests
						return
					}
					log.Printf("%s Tests finished successfully.", prefix)
				}

				// If interactive shell is requested
				if *archShell {
					// Shell mode is not parallelized as it's interactive
					log.Printf("Skipping interactive shell for %s in parallel mode.", distro)
				}
			}(distroName, colorPalette[i%len(colorPalette)])
		}

		wg.Wait()
		close(errs)

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
			log.Println("==> All tests passed successfully.")
		}
		os.Exit(0)
	} else {
		if !quietMode {
			log.Println("==> Some tests failed.")
		}
	}
	os.Exit(1)
}
