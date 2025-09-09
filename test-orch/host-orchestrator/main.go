package main

import (
	"bufio"
	"context"
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strings"
	"sync"
	"time"

	"github.com/docker/docker/client"
	"github.com/fatih/color"

	"host-orchestrator/dockerutil"
)

// SuiteResult captures a single YAML suite outcome parsed from container stdout logs.
type SuiteResult struct {
	Name   string `json:"name"`
	Status string `json:"status"`      // pass | fail
	Expect string `json:"expect,omitempty"` // xfail when expected failure occurred
}

// LogPaths groups stdout/stderr paths for a container run.
type LogPaths struct {
	Stdout string `json:"stdout"`
	Stderr string `json:"stderr"`
}

// ContainerSummary is emitted per-container alongside logs.
type ContainerSummary struct {
	Distro      string        `json:"distro"`
	ImageDigest string        `json:"image_digest"`
	ContainerID string        `json:"container_id"`
	StartedAt   string        `json:"started_at"`
	FinishedAt  string        `json:"finished_at"`
	ExitCode    int           `json:"exit_code"`
	LogPaths    LogPaths      `json:"log_paths"`
	Suites      []SuiteResult `json:"suites"`
}

// parseSuitesFromLog scans a stdout log for PASS/FAIL lines and returns per-suite results.
func parseSuitesFromLog(logPath string) []SuiteResult {
	var out []SuiteResult
	if logPath == "" { return out }
	f, err := os.Open(logPath)
	if err != nil { return out }
	defer f.Close()
	seen := make(map[string]int)
	scanner := bufio.NewScanner(f)
	for scanner.Scan() {
		line := scanner.Text()
		s := strings.TrimSpace(line)
		// 1) "[i/n] PASS suite: <name>"
		if strings.Contains(s, " PASS suite: ") {
			name := after(s, " PASS suite: ")
			if idx := seen[name]; idx == 0 {
				out = append(out, SuiteResult{Name: name, Status: "pass"})
				seen[name] = len(out)
			}
			continue
		}
		// 2) "[i/n] FAIL suite: <name>"
		if strings.Contains(s, " FAIL suite: ") {
			name := after(s, " FAIL suite: ")
			if idx := seen[name]; idx == 0 {
				out = append(out, SuiteResult{Name: name, Status: "fail"})
				seen[name] = len(out)
			} else {
				out[idx-1].Status = "fail"
			}
			continue
		}
		// 3) "Expected failure occurred for suite: <name>"
		if strings.Contains(s, "Expected failure occurred for suite: ") {
			name := after(s, "Expected failure occurred for suite: ")
			if idx := seen[name]; idx == 0 {
				out = append(out, SuiteResult{Name: name, Status: "pass", Expect: "xfail"})
				seen[name] = len(out)
			} else {
				out[idx-1].Status = "pass"
				out[idx-1].Expect = "xfail"
			}
			continue
		}
		// 4) "Suite was expected to fail but passed: <name>"
		if strings.Contains(s, "Suite was expected to fail but passed: ") {
			name := after(s, "Suite was expected to fail but passed: ")
			if idx := seen[name]; idx == 0 {
				out = append(out, SuiteResult{Name: name, Status: "fail", Expect: "xfail"})
				seen[name] = len(out)
			} else {
				out[idx-1].Status = "fail"
				out[idx-1].Expect = "xfail"
			}
			continue
		}
		// 5) "Suite passed (expected pass): <name>"
		if strings.Contains(s, "Suite passed (expected pass): ") {
			name := after(s, "Suite passed (expected pass): ")
			if idx := seen[name]; idx == 0 {
				out = append(out, SuiteResult{Name: name, Status: "pass"})
				seen[name] = len(out)
			} else {
				out[idx-1].Status = "pass"
			}
			continue
		}
		// 6) "Suite failed (expected pass): <name>"
		if strings.Contains(s, "Suite failed (expected pass): ") {
			name := after(s, "Suite failed (expected pass): ")
			if idx := seen[name]; idx == 0 {
				out = append(out, SuiteResult{Name: name, Status: "fail"})
				seen[name] = len(out)
			} else {
				out[idx-1].Status = "fail"
			}
			continue
		}
	}
	return out
}

func after(s, sep string) string {
	i := strings.Index(s, sep)
	if i < 0 { return s }
	return strings.TrimSpace(s[i+len(sep):])
}

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
		failFast    = flag.Bool("fail-fast", true, "Cancel remaining runs on first failure")
		retries     = flag.Int("retries", 2, "Number of retries for docker run start failures or transient errors")
		backoff     = flag.Duration("backoff", 8*time.Second, "Initial backoff between retries (exponential)")
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
		summaries := make(chan ContainerSummary, len(distroList))

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

		// Stable runID used in container names across all goroutines
		runID := time.Now().UTC().Format("20060102-150405Z")
		// Shared docker client used for image inspect calls (safe for concurrent use)
		dockerCli, _ := client.NewClientWithOpts(client.FromEnv, client.WithAPIVersionNegotiation())

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
					// Retry with backoff on failures up to --retries
					var meta dockerutil.RunMeta
					var runErr error
					attempt := 0
					delay := *backoff
					for {
						attempt++
						meta, runErr = dockerutil.RunArchContainer(ctx, distroImageTag, rootDir, "internal-runner", envVars, *keepCtr, *timeout, selected, d, col, runID)
						// Always write per-container summary regardless of success
						digest := ""
						if dockerCli != nil {
							if dg, err := dockerutil.ImageDigest(context.Background(), dockerCli, distroImageTag); err == nil {
								digest = dg
							}
						}
						suites := parseSuitesFromLog(meta.StdoutLogPath)
						sum := ContainerSummary{
							Distro:      d,
							ImageDigest: digest,
							ContainerID: meta.ContainerID,
							StartedAt:   meta.StartedAt.Format(time.RFC3339),
							FinishedAt:  meta.FinishedAt.Format(time.RFC3339),
							ExitCode:    meta.ExitCode,
							LogPaths:    LogPaths{Stdout: meta.StdoutLogPath, Stderr: meta.StderrLogPath},
							Suites:      suites,
						}
						// Persist summary adjacent to logs
						if meta.StdoutLogPath != "" {
							sumDir := filepath.Dir(meta.StdoutLogPath)
							sumPath := filepath.Join(sumDir, fmt.Sprintf("%s-summary.json", meta.ContainerName))
							if f, err := os.Create(sumPath); err == nil {
								_ = json.NewEncoder(f).Encode(sum)
								f.Close()
							}
						}
						summaries <- sum

						if runErr == nil {
							if dockerutil.Allowed(selected, dockerutil.V1) {
								log.Printf("%s Tests finished successfully.", col.Sprint(dockerutil.Prefix(d, dockerutil.V1, "HOST")))
							}
							break
						}
						if attempt > *retries || ctx.Err() != nil {
							errs <- fmt.Errorf("%s docker run failed after %d attempt(s): %w", distroTag, attempt, runErr)
							if *failFast { cancel() }
							break
						}
						if dockerutil.Allowed(selected, dockerutil.V1) {
							log.Printf("%s Retry attempt %d in %s due to error: %v", col.Sprint(dockerutil.Prefix(d, dockerutil.V1, "HOST")), attempt, delay, runErr)
						}
						time.Sleep(delay)
						delay = time.Duration(float64(delay) * 1.6)
						if delay > 60*time.Second { delay = 60 * time.Second }
					}
				}
			}(distroName, colorPalette[i%len(colorPalette)])
		}

		wg.Wait()
		// Close the error channel after all goroutines have finished sending
		close(errs)
		close(summaries)

		for err := range errs {
			if err != nil {
				warn(err.Error())
				ok = false
			}
		}

		// Optionally, write an aggregated summary file ordered by distro
		if !quietMode {
			var all []ContainerSummary
			for s := range summaries { all = append(all, s) }
			// Deterministic order
			sort.Slice(all, func(i, j int) bool { return all[i].Distro < all[j].Distro })
			if len(all) > 0 {
				// Place in repoRoot/logs/aggregate-<runID>.json
				repoRoot, _ := detectRepoRoot()
				aggDir := filepath.Join(repoRoot, "logs")
				_ = os.MkdirAll(aggDir, 0o755)
				aggPath := filepath.Join(aggDir, fmt.Sprintf("aggregate-%s.json", runID))
				if f, err := os.Create(aggPath); err == nil {
					_ = json.NewEncoder(f).Encode(all)
					f.Close()
					hostLog(dockerutil.V1, "Aggregate summary written to %s", aggPath)
				}
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
