package setup

import (
	"fmt"
	"os"

	"container-runner/util"
)

// installDependencies installs base packages required for building and running tests.
func installDependencies() error {
	// CachyOS has a problematic extra repo file that can interfere with finding AUR packages.
	// We remove it to ensure a clean, standard Arch environment for testing.
	_ = os.Remove("/var/lib/pacman/sync/cachyos-extra.db")

	if err := util.RunCmd("pacman", "-Syy", "--noconfirm"); err != nil {
		return fmt.Errorf("pacman sync failed: %w", err)
	}

	packages := []string{"base-devel", "sudo", "git", "curl", "rustup", "which", "findutils"}
	args := append([]string{"-S", "--noconfirm", "--needed"}, packages...)
	if err := util.RunCmd("pacman", args...); err != nil {
		return fmt.Errorf("failed to install packages: %w", err)
	}

	return nil
}
