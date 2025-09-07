package setup

import (
	"fmt"
	"log"
	"os"
	"os/exec"

	"container-runner/util"
)

// setupUsers ensures the 'builder' and 'spread' users exist and sudoers is configured.
func setupUsers() error {
	if err := util.RunCmd("id", "-u", "builder"); err != nil {
		if err := util.RunCmd("useradd", "-m", "builder"); err != nil {
			return fmt.Errorf("failed to create builder user: %w", err)
		}
	}

	// When using persistent mounts, /home/builder may pre-exist and be owned by root.
	// Ensure the builder user's home directory is owned by builder so tools like rustup can write ~/.rustup
	if err := util.RunCmd("chown", "-R", "builder:builder", "/home/builder"); err != nil {
		return fmt.Errorf("failed to set ownership on /home/builder: %w", err)
	}

	sudoersFile := "/etc/sudoers.d/99-builder"
	content := []byte("builder ALL=(ALL) NOPASSWD: ALL\n")
	if err := os.WriteFile(sudoersFile, content, 0440); err != nil {
		return fmt.Errorf("failed to write sudoers file: %w", err)
	}

	if err := util.RunCmd("id", "-u", "spread"); err != nil {
		if err := util.RunCmd("useradd", "-m", "spread"); err != nil {
			return fmt.Errorf("failed to create spread user: %w", err)
		}
	}
	return nil
}

// installAurHelper ensures that an AUR helper (paru) is installed in the container.
func installAurHelper() error {
	// Check multiple ways to detect paru (some distros have it pre-installed)
	paruFound := false
	if _, err := exec.LookPath("paru"); err == nil {
		paruFound = true
	} else if err := util.RunCmd("which", "paru"); err == nil {
		paruFound = true
	} else if err := util.RunCmd("paru", "--version"); err == nil {
		paruFound = true
	}
	
	if paruFound {
		log.Println("paru is already installed.")
		return nil
	}

	log.Println("paru not found, installing from AUR...")
	// Install dependencies for building packages
	if err := util.RunCmd("pacman", "-S", "--noconfirm", "--needed", "base-devel", "git"); err != nil {
		return fmt.Errorf("failed to install base-devel/git: %w", err)
	}
	// As root, create and set permissions for the build directory
	if err := util.RunCmd("mkdir", "-p", "/home/builder/build"); err != nil {
		return err
	}
	if err := util.RunCmd("chown", "-R", "builder:builder", "/home/builder/build"); err != nil {
		return err
	}

	// Clean up any previous partial clones to make this step idempotent
	_ = util.RunCmd("su", "-", "builder", "-c", "rm -rf /home/builder/build/paru-bin")

	// As the non-root 'builder' user, clone and build paru
	// Use || true to handle the case where paru-bin already exists (persistent cache)
	buildCmd := "cd /home/builder/build && (git clone https://aur.archlinux.org/paru-bin.git || true) && cd paru-bin && git pull --rebase && makepkg -s --noconfirm -f"
	if err := util.RunCmd("su", "-", "builder", "-c", buildCmd); err != nil {
		return fmt.Errorf("failed to build paru: %w", err)
	}

	// As root, install the built package (wildcard for version)
	installCmd := "pacman -U --noconfirm /home/builder/build/paru-bin/paru-bin-*.pkg.tar.zst"
	if err := util.RunCmd("sh", "-c", installCmd); err != nil {
		return fmt.Errorf("failed to install paru package: %w", err)
	}
	return nil
}
