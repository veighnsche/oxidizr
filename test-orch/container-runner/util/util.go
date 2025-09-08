package util

import (
	"bufio"
	"fmt"
	"log"
	"os"
	"os/exec"

	"container-runner/analytics"
)

// RunCmd executes a command and streams its output to stdout/stderr.
func RunCmd(name string, args ...string) error {
	log.Printf("Running command: %s %v", name, args)
	cmd := exec.Command(name, args...)
	stdout, _ := cmd.StdoutPipe()
	stderr, _ := cmd.StderrPipe()
	if err := cmd.Start(); err != nil {
		return err
	}
	doneCh := make(chan struct{}, 2)
	go func() {
		scanner := bufio.NewScanner(stdout)
		for scanner.Scan() {
			line := scanner.Text()
			analytics.ProcessLine(line)
			fmt.Fprintln(os.Stdout, line)
		}
		doneCh <- struct{}{}
	}()
	go func() {
		scanner := bufio.NewScanner(stderr)
		for scanner.Scan() {
			line := scanner.Text()
			analytics.ProcessLine(line)
			fmt.Fprintln(os.Stderr, line)
		}
		doneCh <- struct{}{}
	}()
	// Wait for pipes to drain and command to exit
	<-doneCh
	<-doneCh
	return cmd.Wait()
}

// RunCmdQuiet executes a command but does not stream its output.
func RunCmdQuiet(name string, args ...string) error {
	cmd := exec.Command(name, args...)
	stdout, _ := cmd.StdoutPipe()
	stderr, _ := cmd.StderrPipe()
	if err := cmd.Start(); err != nil {
		return err
	}
	doneCh := make(chan struct{}, 2)
	go func() {
		scanner := bufio.NewScanner(stdout)
		for scanner.Scan() {
			analytics.ProcessLine(scanner.Text())
		}
		doneCh <- struct{}{}
	}()
	go func() {
		scanner := bufio.NewScanner(stderr)
		for scanner.Scan() {
			analytics.ProcessLine(scanner.Text())
		}
		doneCh <- struct{}{}
	}()
	<-doneCh
	<-doneCh
	return cmd.Wait()
}

// Has reports whether a command exists in PATH.
func Has(name string) bool {
	_, err := exec.LookPath(name)
	return err == nil
}
