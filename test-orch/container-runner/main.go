package main

import (
	"flag"
	"log"
	"os"
)

func main() {
	// Support both direct invocation and the legacy 'internal-runner' token.
	if len(os.Args) > 1 && os.Args[1] == "internal-runner" {
		// Remove the token so flags can be parsed normally.
		os.Args = append([]string{os.Args[0]}, os.Args[2:]...)
	}

	testFilter := flag.String("test-filter", "", "Run only the named YAML suite directory (e.g., disable-in-german)")
	fullMatrix := flag.Bool("full-matrix", false, "Fail on skipped suites (sets FULL_MATRIX=1)")
	flag.Parse()

	if *testFilter != "" {
		_ = os.Setenv("TEST_FILTER", *testFilter)
	}
	if *fullMatrix {
		_ = os.Setenv("FULL_MATRIX", "1")
	}

	if err := runInContainer(); err != nil {
		log.Fatalf("in-container runner failed: %v", err)
	}
}
