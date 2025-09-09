package yamlrunner

import (
	"bufio"
	"fmt"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"sort"

	"gopkg.in/yaml.v3"

	"container-runner/util"
)

// Task represents the structure of a task.yaml file.
type Task struct {
	Summary     string   `yaml:"summary"`
	Execute     string   `yaml:"execute"`
	Restore     string   `yaml:"restore,omitempty"`
	DistroCheck []string `yaml:"distro-check,omitempty"`
	// Optional expected outcome of the suite's execute block.
	// Valid values: "pass" (default), "fail" (or "xfail").
	// When set to fail/xfail, a non-zero exit from the execute block counts as PASS.
	Expect      string   `yaml:"expect,omitempty"`
}

// Run finds, parses, and executes all task.yaml test suites.
func Run() error {
	projectDir := "/workspace"
	testsDir := filepath.Join(projectDir, "tests")

	var tasks []string
	// POLICY: Do NOT skip or exclude any subdirectories under tests/ during discovery.
	// Skips are prohibited by project policy. If something under tests/ is not a real test,
	// delete or MOVE it out of tests/ instead of masking via code.
	// See WHY_LLMS_ARE_STUPID.md and TESTING_POLICY.md for details.
	err := filepath.Walk(testsDir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if !info.IsDir() && info.Name() == "task.yaml" {
			tasks = append(tasks, path)
		}
		return nil
	})

	if err != nil {
		return fmt.Errorf("failed to find task files: %w", err)
	}

	sort.Strings(tasks)

	// v2 context: list a few discovered suites for debugging
	if len(tasks) > 0 {
		maxList := 5
		if len(tasks) < maxList { maxList = len(tasks) }
		for i := 0; i < maxList; i++ {
			log.Printf("CTX> discovered suite: %s", filepath.Dir(tasks[i]))
		}
		if len(tasks) > maxList {
			log.Printf("CTX> ... and %d more", len(tasks)-maxList)
		}
	}

	testFilter := os.Getenv("TEST_FILTER")
	if testFilter != "" {
		var filteredTasks []string
		for _, taskPath := range tasks {
			if filepath.Base(filepath.Dir(taskPath)) == testFilter {
				filteredTasks = append(filteredTasks, taskPath)
				log.Printf("CTX> filter matched: %s", taskPath)
			}
		}
		if len(filteredTasks) == 0 {
			return fmt.Errorf("test filter '%s' did not match any discovered suites", testFilter)
		}
		tasks = filteredTasks
		log.Printf("Applying filter: running 1 suite ('%s')", testFilter)
	} else {
		log.Printf("Discovered %d YAML test suite(s)", len(tasks))
	}

	for i, taskPath := range tasks {
		suiteName := filepath.Base(filepath.Dir(taskPath))
		log.Printf("[%d/%d] START suite: %s", i+1, len(tasks), suiteName)

		log.Printf("CTX> running suite path: %s", taskPath)
		err := runSingleSuite(taskPath, projectDir)
		if err != nil {
			log.Printf("[%d/%d] ❌ FAIL suite: %s", i+1, len(tasks), suiteName)
			return err
		}
		log.Printf("[%d/%d] ✅ PASS suite: %s", i+1, len(tasks), suiteName)
	}

	return nil
}

func runSingleSuite(taskPath, projectDir string) error {
	content, err := os.ReadFile(taskPath)
	if err != nil {
		return fmt.Errorf("failed to read task file %s: %w", taskPath, err)
	}

	var task Task
	if err := yaml.Unmarshal(content, &task); err != nil {
		return fmt.Errorf("failed to parse YAML from %s: %w", taskPath, err)
	}

	// Check if test is compatible with the current distro
	shouldRun, err := util.ShouldRunOnDistro(task.DistroCheck)
	if err != nil {
		return fmt.Errorf("distro compatibility check failed for %s: %w", filepath.Base(taskPath), err)
	}
	if !shouldRun {
		return fmt.Errorf("suite %s is not compatible with this distro", filepath.Base(taskPath))
	}

	defer func() {
		if task.Restore != "" {
			log.Println("--- Running restore block ---")
			if err := executeScriptBlock(task.Restore, projectDir); err != nil {
				log.Printf("Warning: restore block for %s failed: %v", taskPath, err)
			}
		}
	}()

	if task.Execute != "" {
		log.Println("--- Running execute block ---")
		execErr := executeScriptBlock(task.Execute, projectDir)
		// Interpret outcome based on optional Expect field
		expectFail := strings.EqualFold(task.Expect, "fail") || strings.EqualFold(task.Expect, "xfail")
		suiteName := filepath.Base(filepath.Dir(taskPath))
		if expectFail {
			if execErr != nil {
				log.Printf("✅ Expected failure occurred for suite: %s", suiteName)
				return nil
			}
			// expected to fail but passed
			log.Printf("❌ Suite was expected to fail but passed: %s", suiteName)
			return fmt.Errorf("suite %s expected to fail but passed", suiteName)
		}
		// Default expectation: pass
		if execErr != nil {
			log.Printf("❌ Suite failed (expected pass): %s (err: %v)", suiteName, execErr)
			return execErr
		}
		log.Printf("✅ Suite passed (expected pass): %s", suiteName)
		return nil
	}

	return nil
}

func executeScriptBlock(script, workDir string) error {
	// Create temp file with secure permissions from the start
	tmpDir := os.TempDir()
	tmpFile, err := os.CreateTemp(tmpDir, "task-*.sh")
	if err != nil {
		return fmt.Errorf("failed to create temp script file: %w", err)
	}
	defer os.Remove(tmpFile.Name())

	// Set permissions before writing content to avoid race condition
	if err := os.Chmod(tmpFile.Name(), 0700); err != nil {
		tmpFile.Close()
		return fmt.Errorf("failed to set script permissions: %w", err)
	}

	// Write the script with a strict prelude and traps for better visibility on failures
    // - ERR trap: prints a red X and the failing command/line (when not masked by conditionals)
    // - EXIT trap: prints a red X and exit code if the script exits non-zero (covers e.g. exit 1 in blocks)
    scriptContent := fmt.Sprintf(`#!/usr/bin/env bash
set -Eeuo pipefail
on_err() { echo "❌ Test script failed at line $LINENO: $BASH_COMMAND" >&2; }
on_exit() { local ec=$?; if [ $ec -ne 0 ]; then echo "❌ Test script exited with code $ec" >&2; fi; }
trap on_err ERR
trap on_exit EXIT

%s`, script)
	if _, err := tmpFile.WriteString(scriptContent); err != nil {
		tmpFile.Close()
		return fmt.Errorf("failed to write to temp script: %w", err)
	}
	tmpFile.Close()

	cmd := exec.Command(tmpFile.Name())
	cmd.Dir = workDir
	// Stream with explicit prefixes to distinguish from runner logs
	stdout, _ := cmd.StdoutPipe()
	stderr, _ := cmd.StderrPipe()

	// Run each test script under English locale by default; inline VAR=... command
	// settings inside the script still take precedence for that subcommand only.
	baseEnv := os.Environ()
	baseEnv = setOrReplaceEnv(baseEnv, "LANG", "en_US.UTF-8")
	baseEnv = setOrReplaceEnv(baseEnv, "LC_ALL", "en_US.UTF-8")
	baseEnv = setOrReplaceEnv(baseEnv, "LANGUAGE", "en_US.UTF-8")
	cmd.Env = baseEnv

	if err := cmd.Start(); err != nil {
		return err
	}
	doneCh := make(chan struct{}, 2)
	go func() {
		scanner := bufio.NewScanner(stdout)
		for scanner.Scan() {
			fmt.Fprintln(os.Stdout, scanner.Text())
		}
		doneCh <- struct{}{}
	}()
	go func() {
		scanner := bufio.NewScanner(stderr)
		for scanner.Scan() {
			fmt.Fprintln(os.Stderr, scanner.Text())
		}
		doneCh <- struct{}{}
	}()
	<-doneCh
	<-doneCh
	return cmd.Wait()
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
