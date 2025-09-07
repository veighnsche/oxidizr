package setup

import (
	"fmt"
	"log"
	"os"
	"os/exec"
	"path/filepath"

	"container-runner/util"
)

// Run prepares the container for tests, mirroring the setup steps from the old entrypoint.sh.
func Run() error {
	log.Println("==> Staging workspace...")
	if err := stageWorkspace(); err != nil {
		return err
	}

	log.Println("==> Installing system dependencies...")
	if err := installDependencies(); err != nil {
		return err
	}

	log.Println("==> Setting up users...")
	if err := setupUsers(); err != nil {
		return err
	}

	log.Println("==> Installing AUR helper...")
	if err := installAurHelper(); err != nil {
		return err
	}

	log.Println("==> Setting up Rust toolchain...")
	if err := setupRust(); err != nil {
		return err
	}

	log.Println("==> Building project...")
	if err := buildProject(); err != nil {
		return err
	}

	return nil
}

func stageWorkspace() error {
	projectDir := "/root/project/oxidizr-arch"
	if err := os.MkdirAll(projectDir, 0755); err != nil {
		return fmt.Errorf("failed to create project directory: %w", err)
	}
	return util.RunCmd("cp", "-a", "/workspace/.", projectDir)
}

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

func setupUsers() error {
	if err := util.RunCmd("id", "-u", "builder"); err != nil {
		if err := util.RunCmd("useradd", "-m", "builder"); err != nil {
			return fmt.Errorf("failed to create builder user: %w", err)
		}
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

func installAurHelper() error {
	cmd := `mkdir -p ~/build && cd ~/build && git clone https://aur.archlinux.org/paru-bin.git || true && cd paru-bin && makepkg -si --noconfirm`
	return util.RunCmd("su", "-", "builder", "-c", cmd)
}

func setupRust() error {
	log.Println("==> Setting up Rust toolchain...")
	util.RunCmd("rustup", "default", "stable")
	// Also set for builder user
	util.RunCmd("su", "-", "builder", "-c", "rustup default stable")

	// Ensure an AUR helper is installed (paru)
	log.Println("==> Ensuring AUR helper (paru) is installed...")
	if _, err := exec.LookPath("paru"); err != nil {
		log.Println("paru not found, installing from AUR...")
		// Install dependencies for building packages
		util.RunCmd("pacman", "-S", "--noconfirm", "--needed", "base-devel", "git")
		// As root, create and set permissions for the build directory
		util.RunCmd("mkdir", "-p", "/home/builder/build")
		util.RunCmd("chown", "-R", "builder:builder", "/home/builder/build")

		// As the non-root 'builder' user, clone and build paru
		buildCmd := "cd /home/builder/build && git clone https://aur.archlinux.org/paru-bin.git && cd paru-bin && makepkg -s --noconfirm"
		util.RunCmd("su", "-", "builder", "-c", buildCmd)

		// As root, install the built package
		// Note: The exact package version might change, so we use a wildcard.
		installCmd := "pacman -U --noconfirm /home/builder/build/paru-bin/paru-bin-*.pkg.tar.zst"
		util.RunCmd("sh", "-c", installCmd)
	} else {
		log.Println("paru is already installed.")
	}
	return nil
}

func buildProject() error {
	projectDir := "/root/project/oxidizr-arch"
	if _, err := os.Stat(filepath.Join(projectDir, "Cargo.toml")); os.IsNotExist(err) {
		return fmt.Errorf("Cargo.toml not found in %s", projectDir)
	}

	originalDir, _ := os.Getwd()
	os.Chdir(projectDir)
	defer os.Chdir(originalDir)

	buildJobs := os.Getenv("CARGO_BUILD_JOBS")
	if buildJobs == "" {
		buildJobs = "2"
	}
	if err := util.RunCmd("cargo", "build", "--release", "-j", buildJobs); err != nil {
		return fmt.Errorf("cargo build failed: %w", err)
	}

	sourcePath := filepath.Join(projectDir, "target/release/oxidizr-arch")
	destPath := "/usr/local/bin/oxidizr-arch"
	if err := util.RunCmd("install", "-m", "0755", sourcePath, destPath); err != nil {
		return fmt.Errorf("failed to install binary: %w", err)
	}

	return util.RunCmdQuiet("oxidizr-arch", "--help")
}
