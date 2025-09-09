package dockerutil

import (
	"fmt"
	"io"
	"strings"

	"github.com/fatih/color"
)

// Verb represents intrinsic verbosity level v0..v3.
type Verb int

const (
	V0 Verb = 0 // critical/summary
	V1 Verb = 1 // default lifecycle
	V2 Verb = 2 // verbose
	V3 Verb = 3 // very verbose / trace
)

// Allowed implements the filtering table from VERBOSITY.md.
// A message at level lvl is printed if lvl <= selected.
func Allowed(selected, lvl Verb) bool { return lvl <= selected }

// Prefix builds a canonical prefix string: "[distro][vN][SCOPE]".
// Scope may be "" (omitted) to yield "[distro][vN]".
func Prefix(distro string, lvl Verb, scope string) string {
	b := &strings.Builder{}
	if distro != "" {
		fmt.Fprintf(b, "[%s]", distro)
	}
	fmt.Fprintf(b, "[v%d]", int(lvl))
	if scope != "" {
		fmt.Fprintf(b, "[%s]", strings.Trim(scope, "[]"))
	}
	return b.String()
}

// prefixWriter is a helper to prepend a computed prefix to each line of output.
// Used to colorize and prefix Docker build/run output in verbose mode.
// The raw JSON stream is still fully parsed for error detection even when not verbose.
type prefixWriter struct {
	distro string
	lvl    Verb
	scope  string // e.g. "HOST" or ""
	w      io.Writer
	col    *color.Color
}

func (pw *prefixWriter) Write(p []byte) (n int, err error) {
	s := string(p)
	// Simple case: if no newline, just write the bytes unmodified
	if !strings.Contains(s, "\n") {
		return pw.w.Write(p)
	}

	pfx := Prefix(pw.distro, pw.lvl, pw.scope)
	// Split lines and prepend prefix
	lines := strings.Split(strings.TrimRight(s, "\n"), "\n")
	for _, line := range lines {
		if pw.col != nil {
			fmt.Fprintf(pw.w, "%s %s\n", pw.col.Sprint(pfx), line)
		} else {
			fmt.Fprintf(pw.w, "%s %s\n", pfx, line)
		}
	}
	return len(p), nil
}
