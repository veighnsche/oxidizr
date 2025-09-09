package dockerutil

import (
	"encoding/json"
	"os"
	"path/filepath"
	"sync"
	"time"
)

// HostJSONLLogger writes host-side lifecycle events in a canonical JSONL envelope.
// Fields: ts, component, level, run_id, container_id, distro, stage, event, message, rc, duration_ms

type HostJSONLLogger struct {
	path        string
	runID       string
	distro      string
	containerID string
	mu          sync.Mutex
}

func NewHostJSONLLogger(dir, runID, distro string) *HostJSONLLogger {
	_ = os.MkdirAll(dir, 0o755)
	return &HostJSONLLogger{path: filepath.Join(dir, "host.jsonl"), runID: runID, distro: distro}
}

func (l *HostJSONLLogger) SetContainerID(cid string) {
	l.mu.Lock()
	l.containerID = cid
	l.mu.Unlock()
}

func (l *HostJSONLLogger) Event(level, stage, eventName, message string, rc *int, durationMs *int64) {
	l.mu.Lock()
	defer l.mu.Unlock()
	rec := map[string]any{
		"ts":           time.Now().UTC().Format(time.RFC3339Nano),
		"component":    "host",
		"level":        level,
		"run_id":       l.runID,
		"container_id": l.containerID,
		"distro":       l.distro,
		"stage":        stage,
		"event":        eventName,
	}
	if message != "" {
		rec["message"] = message
	}
	if rc != nil {
		rec["rc"] = *rc
	}
	if durationMs != nil {
		rec["duration_ms"] = *durationMs
	}
	f, err := os.OpenFile(l.path, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0o644)
	if err != nil {
		return
	}
	defer f.Close()
	_ = json.NewEncoder(f).Encode(rec)
}
