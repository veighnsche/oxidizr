package yamlrunner

import (
	"fmt"
	"log"
	"os"
	"os/exec"
	"path/filepath"
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
}

// Run finds, parses, and executes all task.yaml test suites.
func Run() error {
	projectDir := "/root/project/oxidizr-arch"
	testsDir := filepath.Join(projectDir, "tests")

	var tasks []string
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

	testFilter := os.Getenv("TEST_FILTER")
	if testFilter != "" {
		var filteredTasks []string
		for _, taskPath := range tasks {
			if filepath.Base(filepath.Dir(taskPath)) == testFilter {
				filteredTasks = append(filteredTasks, taskPath)
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

	fullMatrix := os.Getenv("FULL_MATRIX") == "1"

	for i, taskPath := range tasks {
		suiteName := filepath.Base(filepath.Dir(taskPath))
		log.Printf("[%d/%d] START suite: %s", i+1, len(tasks), suiteName)

		skipped, err := runSingleSuite(taskPath, projectDir)
		if err != nil {
			log.Printf("[%d/%d] FAIL suite: %s", i+1, len(tasks), suiteName)
			return err
		}
		if skipped {
			log.Printf("[%d/%d] SKIP suite: %s", i+1, len(tasks), suiteName)
			if fullMatrix {
				return fmt.Errorf("suite '%s' was skipped in FULL_MATRIX mode", suiteName)
			}
			continue
		}
		log.Printf("[%d/%d] PASS suite: %s", i+1, len(tasks), suiteName)
	}

	return nil
}

func runSingleSuite(taskPath, projectDir string) (bool, error) {
	content, err := os.ReadFile(taskPath)
	if err != nil {
		return false, fmt.Errorf("failed to read task file %s: %w", taskPath, err)
	}

	var task Task
	if err := yaml.Unmarshal(content, &task); err != nil {
		return false, fmt.Errorf("failed to parse YAML from %s: %w", taskPath, err)
	}

	// Check if test is compatible with the current distro
	shouldRun, err := util.ShouldRunOnDistro(task.DistroCheck)
	if err != nil {
		log.Printf("SKIPPING suite %s: could not check distro compatibility: %v", filepath.Base(taskPath), err)
		return true, nil // skipped
	}
	if !shouldRun {
		log.Printf("SKIPPING suite %s: not compatible with this distro", filepath.Base(taskPath))
		return true, nil // skipped
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
		return false, executeScriptBlock(task.Execute, projectDir)
	}

	return false, nil
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

	// Write the script as-is; environment setup ensures required tools are present
	scriptContent := fmt.Sprintf("#!/usr/bin/env bash\nset -euo pipefail\n\n%s", script)
	if _, err := tmpFile.WriteString(scriptContent); err != nil {
		tmpFile.Close()
		return fmt.Errorf("failed to write to temp script: %w", err)
	}
	tmpFile.Close()

	cmd := exec.Command(tmpFile.Name())
	cmd.Dir = workDir
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	return cmd.Run()
}
