package setup

import (
	"log"
	"container-runner/util"
)

// setupRust ensures the Rust toolchain default is set for root and the builder user.
func setupRust() error {
	log.Println("==> Setting up Rust toolchain...")
	// rustup for root
	_ = util.RunCmd("rustup", "default", "stable")
	// Also set for builder user
	_ = util.RunCmd("su", "-", "builder", "-c", "rustup default stable")
	// AUR helper installation is handled in installAurHelper()
	return nil
}
