package setup

import (
	"log"
	"os/exec"

	"container-runner/util"
)

// setupRust ensures rust toolchain is available and an AUR helper (paru) is present.
func setupRust() error {
	log.Println("==> Setting up Rust toolchain...")
	_ = util.RunCmd("rustup", "default", "stable")
	// Also set for builder user
	_ = util.RunCmd("su", "-", "builder", "-c", "rustup default stable")

	// Ensure an AUR helper is installed (paru)
	log.Println("==> Ensuring AUR helper (paru) is installed...")
	if _, err := exec.LookPath("paru"); err != nil {
		log.Println("paru not found, installing from AUR...")
		// Install dependencies for building packages
		if err := util.RunCmd("pacman", "-S", "--noconfirm", "--needed", "base-devel", "git"); err != nil {
			return err
		}
		// As root, create and set permissions for the build directory
		if err := util.RunCmd("mkdir", "-p", "/home/builder/build"); err != nil {
			return err
		}
		if err := util.RunCmd("chown", "-R", "builder:builder", "/home/builder/build"); err != nil {
			return err
		}

		// As the non-root 'builder' user, clone and build paru
		buildCmd := "cd /home/builder/build && git clone https://aur.archlinux.org/paru-bin.git && cd paru-bin && makepkg -s --noconfirm"
		if err := util.RunCmd("su", "-", "builder", "-c", buildCmd); err != nil {
			return err
		}

		// As root, install the built package
		installCmd := "pacman -U --noconfirm /home/builder/build/paru-bin/paru-bin-*.pkg.tar.zst"
		if err := util.RunCmd("sh", "-c", installCmd); err != nil {
			return err
		}
	} else {
		log.Println("paru is already installed.")
	}
	return nil
}
