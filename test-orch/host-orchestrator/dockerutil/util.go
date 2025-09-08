package dockerutil

import (
	"fmt"
	"io"
	"strings"

	"github.com/fatih/color"
)

// prefixWriter is a helper to prepend a prefix to each line of output.
// It is used to colorize and prefix Docker build/run output in verbose mode.
// The raw JSON stream is still fully parsed for error detection even when not verbose.
//
type prefixWriter struct {
	prefix string
	w      io.Writer
	col    *color.Color
}

func (pw *prefixWriter) Write(p []byte) (n int, err error) {
	// Simple case: if no newline, just write the bytes
	if !strings.Contains(string(p), "\n") {
		return pw.w.Write(p)
	}

	// Split lines and prepend prefix
	lines := strings.Split(strings.TrimRight(string(p), "\n"), "\n")
	for _, line := range lines {
		fmt.Fprintf(pw.w, "%s %s\n", pw.col.Sprint(pw.prefix), line)
	}
	return len(p), nil
}
