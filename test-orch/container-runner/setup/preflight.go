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

	// Required core tools in the base image
	required := []string{"pacman", "curl", "git", "locale-gen", "localedef"}
	missing := []string{}
	for _, bin := range required {
		if !util.Has(bin) {
			missing = append(missing, bin)
		}
	}
	if len(missing) > 0 {
		return fmt.Errorf("missing required tools in base image: %v", missing)
	}

	// Basic network sanity (DNS + HTTPS)
	if err := util.RunCmd("sh", "-lc", "curl -sSf https://archlinux.org >/dev/null"); err != nil {
		return fmt.Errorf("network check failed (HTTPS to archlinux.org): %w", err)
	}

	// Surface FULL_MATRIX mode for logs
	if os.Getenv("FULL_MATRIX") == "1" {
		log.Println("Preflight: FULL_MATRIX=1 (skipped suites and infra failures are fatal)")
	}
	return nil
}
