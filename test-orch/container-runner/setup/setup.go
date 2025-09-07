package setup

import (
	"log"
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

	log.Println("==> Setting up locales...")
	if err := setupLocales(); err != nil {
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
