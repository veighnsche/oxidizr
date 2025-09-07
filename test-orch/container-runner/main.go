package main

import (
	"log"
	"os"
)

func main() {
	// This allows the runner to be invoked either as the default entrypoint
	// or with an explicit 'internal-runner' command for clarity.
	if len(os.Args) > 1 && os.Args[1] != "internal-runner" {
		log.Fatalf("unexpected command: %v", os.Args[1:])
	}

	if err := runInContainer(); err != nil {
		log.Fatalf("in-container runner failed: %v", err)
	}
}
