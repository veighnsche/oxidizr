package main

import (
	"bytes"
	"fmt"
	"log"
	"os"
	"os/exec"
	"strings"
	"time"
)

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

func warn(v ...interface{}) {
	log.Println("WARN:", fmt.Sprint(v...))
}

func section(title string) {
	log.Println()
	log.Println("==>", title)
	time.Sleep(10 * time.Millisecond) // keep logs readable
}

func prefixRun() string { return "RUN>" }
