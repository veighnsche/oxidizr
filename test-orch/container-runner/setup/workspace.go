package setup

// stageWorkspace prepares the working directory. We now operate directly in
// /workspace (the bind-mounted repo root) to avoid expensive copies and
// potential path aliasing issues.
func stageWorkspace() error {
    return nil
}
