package setup

import (
	"fmt"
	"log"
	"os"

	"container-runner/util"
)

// preflight performs early sanity checks so we fail fast with actionable messages.
func preflight() error {
	log.Println("==> Preflight checks...")

	// Minimal requirement: we only need pacman to be present before we install dependencies.
	if !util.Has("pacman") {
		return fmt.Errorf("missing required tool in base image: pacman")
	}

	// Basic network sanity (DNS + HTTPS). If curl is not available yet, skip here;
	// it will be installed during dependency setup.
	if util.Has("curl") {
		if err := util.RunCmd("sh", "-lc", "curl -sSf https://archlinux.org >/dev/null"); err != nil {
			return fmt.Errorf("network check failed (HTTPS to archlinux.org): %w", err)
		}
	} else {
		log.Println("Preflight: curl not present yet; skipping network check until after dependencies")
	}

	// Surface FULL_MATRIX mode for logs
	if os.Getenv("FULL_MATRIX") == "1" {
		log.Println("Preflight: FULL_MATRIX=1 (skipped suites and infra failures are fatal)")
	}

	// Probe and summarize environment differences for logs
	if distroID, err := util.CurrentDistroID(); err == nil {
		// Check for German locale presence via file and locale -a list
		_, fileErr := os.Stat("/usr/share/i18n/locales/de_DE")
		filePresent := fileErr == nil
		cmdPresent := util.RunCmdQuiet("sh", "-lc", "locale -a | grep -qi '^de_DE\\.'") == nil

		// Detect AUR helpers commonly found on derivatives
		hasParu := util.Has("paru")
		hasYay := util.Has("yay")

		log.Printf("Preflight summary: distro=%s, de_DE_present=%t (file=%t,locale=%t), aur_paru=%t, aur_yay=%t",
			distroID, (filePresent || cmdPresent), filePresent, cmdPresent, hasParu, hasYay)
	} else {
		log.Println("Preflight summary: distro could not be determined from /etc/os-release")
	}
	return nil
}
