package setup

import (
	"fmt"
	"log"
	"os"
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
	if _, err := os.Stat("/home/builder"); os.IsNotExist(err) {
		if err := util.RunCmd("useradd", "-m", "builder"); err != nil {
			return fmt.Errorf("failed to create builder user: %w", err)
		}
	}

	sudoersFile := "/etc/sudoers.d/99-builder"
	content := []byte("builder ALL=(ALL) NOPASSWD: ALL\n")
	if err := os.WriteFile(sudoersFile, content, 0440); err != nil {
		return fmt.Errorf("failed to write sudoers file: %w", err)
	}

	if _, err := os.Stat("/home/spread"); os.IsNotExist(err) {
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
	if err := util.RunCmd("rustup", "default", "stable"); err != nil {
		log.Printf("Warning: failed to set default rust toolchain for root: %v", err)
	}
	if err := util.RunCmd("su", "-", "builder", "-c", "rustup default stable"); err != nil {
		log.Printf("Warning: failed to set default rust toolchain for builder: %v", err)
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
