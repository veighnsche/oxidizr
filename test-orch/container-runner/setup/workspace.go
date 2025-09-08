package setup

import (
	"fmt"
	"os"
)

// stageWorkspace prepares the working directory. We now operate directly in
// /workspace (the bind-mounted repo root) to avoid expensive copies and
// potential path aliasing issues.
func stageWorkspace() error {
    const mount = "/workspace"
    fi, err := os.Stat(mount)
    if err != nil {
        return fmt.Errorf("missing /workspace mount: %w. Ensure host orchestrator runs docker with -v <repo>:/workspace and that the path exists.", err)
    }
    if !fi.IsDir() {
        return fmt.Errorf("/workspace exists but is not a directory")
    }
    // Check writeability by creating and removing a temp file
    f, err := os.CreateTemp(mount, ".ws-*.probe")
    if err != nil {
        return fmt.Errorf("/workspace not writable (create failed): %w", err)
    }
    name := f.Name()
    _ = f.Close()
    if err := os.Remove(name); err != nil {
        return fmt.Errorf("/workspace write probe cleanup failed: %w", err)
    }
    return nil
}
