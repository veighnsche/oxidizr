package setup

import (
	"fmt"
	"os"

	"container-runner/util"
)

// stageWorkspace copies the mounted repository into an internal working dir.
func stageWorkspace() error {
	projectDir := "/root/project/oxidizr-arch"
	if err := os.MkdirAll(projectDir, 0755); err != nil {
		return fmt.Errorf("failed to create project directory: %w", err)
	}
	return util.RunCmd("cp", "-a", "/workspace/.", projectDir)
}
