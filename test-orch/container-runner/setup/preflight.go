package setup

import (
	"errors"
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
		return fmt.Errorf("missing required tool in base image: pacman. Before adding masking/skip logic, read WHY_LLMS_ARE_STUPID.md to avoid false positives.")
	}

	// Basic network sanity (DNS + HTTPS). If curl is not available yet, skip here;
	// it will be installed during dependency setup.
	if util.Has("curl") {
		if err := util.RunCmd("sh", "-lc", "curl -sSf https://archlinux.org >/dev/null"); err != nil {
			return fmt.Errorf("network check failed (HTTPS to archlinux.org): %w. Before adding masking/skip logic, read WHY_LLMS_ARE_STUPID.md to avoid false positives.", err)
		}
	} else {
		log.Println("CTX> Preflight: curl not present yet; skipping network check until after dependencies")
	}

	// Strict matrix semantics are the default; no special-casing needed.

	// Probe and summarize environment differences for logs
	if distroID, err := util.CurrentDistroID(); err == nil {
		// Check for German locale presence via file and locale -a list
		_, fileErr := os.Stat("/usr/share/i18n/locales/de_DE")
		var filePresent bool
		if fileErr == nil {
			filePresent = true
		} else if errors.Is(fileErr, os.ErrNotExist) {
			filePresent = false
		} else {
			return fmt.Errorf("preflight: failed to stat de_DE locale definition: %w", fileErr)
		}
		cmdPresent := util.RunCmdQuiet("sh", "-lc", "locale -a | grep -qi '^de_DE\\.'") == nil

		// Enforce baked-in German locale (policy): both the definition file and a listed locale must exist
		if !(filePresent && cmdPresent) {
			return fmt.Errorf(
				"preflight: missing German locale (de_DE). filePresent=%t, listedInLocaleA=%t. Expected de_DE.UTF-8 to be baked into the image. Rebuild the image from test-orch/docker/Dockerfile and ensure locale-gen runs at build time. Before adding masking/skip logic, read WHY_LLMS_ARE_STUPID.md to avoid false positives.",
				filePresent, cmdPresent,
			)
		}

		// Detect AUR helpers commonly found on derivatives
		hasParu := util.Has("paru")
		hasYay := util.Has("yay")

		log.Printf("CTX> Preflight summary: distro=%s, de_DE_present=%t (file=%t,locale=%t), aur_paru=%t, aur_yay=%t",
			distroID, (filePresent && cmdPresent), filePresent, cmdPresent, hasParu, hasYay)
	} else {
		log.Println("CTX> Preflight summary: distro could not be determined from /etc/os-release")
	}
	return nil
}
