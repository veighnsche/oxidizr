package dockerutil

import (
	"regexp"
	"strings"
)

// classifyLine infers an intrinsic verbosity class and scope from a line.
// Returns (vLevel 0..3, scope "[RUNNER]" or "", content without leading tags).
func classifyLine(line string) (int, string, string) {
	// Detect explicit runner-tagged lines like "[v2][RUNNER] message"
	if strings.HasPrefix(line, "[v") {
		reRunner := regexp.MustCompile(`^\[v([0-3])\]\[RUNNER\]\s+`)
		if m := reRunner.FindStringSubmatch(line); m != nil {
			lvl := int(m[1][0] - '0')
			content := reRunner.ReplaceAllString(line, "")
			return lvl, "[RUNNER]", content
		}
		// Generic [vN] tag (no scope); treat as product/raw intrinsic level
		reGeneric := regexp.MustCompile(`^\[v([0-3])\]\s+`)
		if m := reGeneric.FindStringSubmatch(line); m != nil {
			lvl := int(m[1][0] - '0')
			content := reGeneric.ReplaceAllString(line, "")
			return lvl, "", content
		}
	}
	if strings.HasPrefix(line, "[RUNNER] ") {
		content := strings.TrimPrefix(line, "[RUNNER] ")
		if strings.HasPrefix(content, "RUN> ") {
			return 2, "[RUNNER]", content
		}
		if strings.HasPrefix(content, "CTX> ") {
			return 2, "[RUNNER]", content
		}
		if strings.HasPrefix(content, "TRC> ") {
			return 3, "[RUNNER]", content
		}
		if strings.Contains(content, "❌") {
			return 0, "[RUNNER]", content
		}
		return 1, "[RUNNER]", content
	}
	// Structured audit lines from Rust product logs are verbose by nature; hide at v1.
	// Example: "INFO ...: audit timestamp=... component=operation event=RESTORE_FILE ..."
	if strings.Contains(line, " audit ") || (strings.Contains(line, " component=") && strings.Contains(line, " event=")) {
		return 3, "", line
	}
	// Detect Rust env_logger style levels inside product output
	// Map: ERROR->v0, WARN->v1, INFO->v1, DEBUG->v2, TRACE->v3
	switch {
	case strings.Contains(line, " ERROR "):
		return 0, "", line
	case strings.Contains(line, " WARN "):
		return 1, "", line
	case strings.Contains(line, " INFO "):
		return 1, "", line
	case strings.Contains(line, " DEBUG "):
		return 2, "", line
	case strings.Contains(line, " TRACE "):
		return 3, "", line
	}
	// Default for container script/stdout lines
	return 1, "", line
}
