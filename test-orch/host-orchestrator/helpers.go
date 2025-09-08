package main

import (
	"bytes"
	"crypto/sha256"
	"fmt"
	"io"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strings"
	"time"
)

// Verbosity controls
// 0 = quiet (only final summary and critical errors)
// 1 = normal (default)
// 2 = verbose (-v)
// 3 = trace (-vv)
var (
	quietMode      bool
	verbosityLevel = 1
)

func setQuiet(q bool) { quietMode = q }

// computeBuildHash walks selected paths under the docker context directory and returns a short
// sha256 hex digest. Any change to inputs yields a new tag, so existing images are reused only
// when inputs are unchanged.
func computeBuildHash(ctxDir string) (string, error) {
    // Inputs that affect the in-container runner image
    inputs := []string{
        filepath.Join(ctxDir, "docker/Dockerfile"),
        filepath.Join(ctxDir, "container-runner"), // binary name if present (ignored if not)
        filepath.Join(ctxDir, "container-runner/"), // source tree
    }
    h := sha256.New()
    seen := make(map[string]bool)
    for _, p := range inputs {
        // Expand directories
        fi, err := os.Stat(p)
        if err != nil {
            continue
        }
        if fi.IsDir() {
            filepath.Walk(p, func(path string, info os.FileInfo, err error) error {
                if err != nil { return nil }
                if info.IsDir() { return nil }
                rel, _ := filepath.Rel(ctxDir, path)
                if seen[rel] { return nil }
                seen[rel] = true
                io.WriteString(h, rel)
                f, err := os.Open(path)
                if err == nil {
                    _, _ = io.Copy(h, f)
                    f.Close()
                }
                return nil
            })
        } else {
            rel, _ := filepath.Rel(ctxDir, p)
            if !seen[rel] {
                seen[rel] = true
                io.WriteString(h, rel)
                f, err := os.Open(p)
                if err == nil {
                    _, _ = io.Copy(h, f)
                    f.Close()
                }
            }
        }
    }
    // Stabilize by hashing the filenames set order too
    var files []string
    for k := range seen { files = append(files, k) }
    sort.Strings(files)
    for _, k := range files { io.WriteString(h, "|"+k) }
    sum := fmt.Sprintf("%x", h.Sum(nil))
    if len(sum) > 12 { sum = sum[:12] }
    return sum, nil
}
func setVerbosity(lvl int) {
	if lvl < 0 { lvl = 0 }
	if lvl > 3 { lvl = 3 }
	verbosityLevel = lvl
}

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

// runMaybeSilent runs a command and only streams output when verbose is true.
func runMaybeSilent(verbose bool, name string, args ...string) error {
	if verbose {
		return run(name, args...)
	}
	return runSilent(name, args...)
}

// isRoot reports whether the current process has UID 0.
func isRoot() bool {
	return os.Geteuid() == 0
}

func warn(v ...interface{}) {
	if quietMode {
		return
	}
	log.Println("WARN:", fmt.Sprint(v...))
}

func section(title string) {
	if quietMode || verbosityLevel < 2 {
		return
	}
	log.Println()
	log.Println("==>", title)
	time.Sleep(10 * time.Millisecond) // keep logs readable
}

func prefixRun() string { return "RUN>" }

// vlog prints when the current verbosity is >= level (0..3)
func vlog(level int, v ...interface{}) {
	if verbosityLevel >= level {
		log.Println(v...)
	}
}

// detectRepoRoot finds the git repository root, or returns an error.
func detectRepoRoot() (string, error) {
	// Prefer `git rev-parse --show-toplevel`
	out, err := exec.Command("git", "rev-parse", "--show-toplevel").Output()
	if err == nil && len(out) > 0 {
		return strings.TrimSpace(string(out)), nil
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
