package util

import (
	"fmt"
	"io"
	"log"
	"os"
	"os/exec"
	"strings"
)

// RunCmd executes a command and streams its output to stdout/stderr.
func RunCmd(name string, args ...string) error {
	log.Printf("Running command: %s %v", name, args)
	var cmd *exec.Cmd
	switch name {
	case "pacman", "paru", "yay":
		fullCmd := fmt.Sprintf("yes | %s %s", name, strings.Join(args, " "))
		cmd = exec.Command("sh", "-c", fullCmd)
	default:
		cmd = exec.Command(name, args...)
	}
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}

// RunCmdQuiet executes a command but does not stream its output.
func RunCmdQuiet(name string, args ...string) error {
	var cmd *exec.Cmd
	switch name {
	case "pacman", "paru", "yay":
		fullCmd := fmt.Sprintf("yes | %s %s", name, strings.Join(args, " "))
		cmd = exec.Command("sh", "-c", fullCmd)
	default:
		cmd = exec.Command(name, args...)
	}
	cmd.Stdout = io.Discard
	cmd.Stderr = io.Discard
	return cmd.Run()
}
