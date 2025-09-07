package util

import (
	"io"
	"log"
	"os"
	"os/exec"
)

// RunCmd executes a command and streams its output to stdout/stderr.
func RunCmd(name string, args ...string) error {
	log.Printf("Running command: %s %v", name, args)
	cmd := exec.Command(name, args...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}

// RunCmdQuiet executes a command but does not stream its output.
func RunCmdQuiet(name string, args ...string) error {
	cmd := exec.Command(name, args...)
	cmd.Stdout = io.Discard
	cmd.Stderr = io.Discard
	return cmd.Run()
}
