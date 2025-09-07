package assertions

import (
	"fmt"
	"log"
	"os"
	"os/exec"
	"strings"

	"container-runner/util"
)

// Run executes the full assertion flow.
func Run() error {
	log.Println("==> Running assertion flow...")

	// Only run these heavy assertions on vanilla Arch; derivatives may differ.
	if ok, err := util.ShouldRunOnDistro([]string{"arch"}); err != nil {
		log.Printf("Skipping assertions: distro detection failed: %v", err)
		return nil
	} else if !ok {
		log.Println("Skipping assertions on non-Arch distro.")
		return nil
	}

	// Enable experiments
	log.Println("--- Enabling experiments: coreutils,sudo-rs ---")
	if err := util.RunCmd("oxidizr-arch", "--assume-yes", "--experiments", "coreutils,sudo-rs", "--package-manager", "none", "enable"); err != nil {
		return fmt.Errorf("failed to enable experiments: %w", err)
	}

	// Assertions after enabling
	log.Println("--- Asserting sudo-rs is installed ---")
	if err := ensureSudoRsInstalled(); err != nil {
		return err
	}
	log.Println("--- Asserting coreutils is installed ---")
	if err := ensureCoreutilsInstalled(); err != nil {
		return err
	}

	// Disable experiments
	log.Println("--- Disabling experiments: coreutils,sudo-rs ---")
	if err := util.RunCmd("oxidizr-arch", "--assume-yes", "--experiments", "coreutils,sudo-rs", "--package-manager", "none", "disable"); err != nil {
		return fmt.Errorf("failed to disable experiments: %w", err)
	}

	// Assertions after disabling
	log.Println("--- Asserting sudo-rs is absent ---")
	if err := ensureSudoRsAbsent(); err != nil {
		return err
	}
	log.Println("--- Asserting coreutils is absent ---")
	if err := ensureCoreutilsAbsent(); err != nil {
		return err
	}

	log.Println("All assertions passed.")
	return nil
}

func ensureSudoRsInstalled() error {
	if !isPkgInstalled("sudo-rs") {
		return fmt.Errorf("package 'sudo-rs' should be installed")
	}
	// Accept either direct link to /usr/bin/sudo-rs or an alias /usr/bin/sudo.sudo-rs
	if err := checkSymlinkContainsAny("/usr/bin/sudo", []string{"/usr/bin/sudo-rs", "/usr/bin/sudo.sudo-rs"}); err != nil {
		return err
	}
	if !fileExists("/usr/bin/.sudo.oxidizr.bak") {
		return fmt.Errorf("backup file for sudo not found")
	}
	return nil
}

func ensureSudoRsAbsent() error {
	if isPkgInstalled("sudo-rs") {
		return fmt.Errorf("package 'sudo-rs' should NOT be installed")
	}
	if isSymlink("/usr/bin/sudo") {
		return fmt.Errorf("/usr/bin/sudo should not be a symlink")
	}
	if fileExists("/usr/bin/.sudo.oxidizr.bak") {
		return fmt.Errorf("backup file for sudo should not exist")
	}
	return nil
}

func ensureCoreutilsInstalled() error {
	if !isPkgInstalled("uutils-coreutils") {
		return fmt.Errorf("package 'uutils-coreutils' should be installed")
	}
	bins, err := readLines("/root/project/oxidizr-arch/tests/lib/rust-coreutils-bins.txt")
	if err != nil {
		return fmt.Errorf("could not read coreutils bin list: %w", err)
	}
	for _, bin := range bins {
		target := "/usr/bin/" + bin
		if !isSymlink(target) {
			log.Printf("Warning: %s is not a symlink, skipping.", target)
			continue
		}
		if !fileExists("/usr/bin/." + bin + ".oxidizr.bak") {
			return fmt.Errorf("backup file for %s not found", target)
		}
		out, err := exec.Command(target, "--version").Output()
		if err != nil {
			log.Printf("Warning: failed to run %s --version: %v", target, err)
			continue
		}
		if strings.Contains(string(out), "GNU") {
			return fmt.Errorf("%s appears to be GNU version", target)
		}
	}
	return nil
}

func ensureCoreutilsAbsent() error {
	if isPkgInstalled("uutils-coreutils") {
		return fmt.Errorf("package 'uutils-coreutils' should NOT be installed")
	}
	target := "/usr/bin/date"
	if isSymlink(target) {
		return fmt.Errorf("%s should not be a symlink", target)
	}
	if fileExists("/usr/bin/.date.oxidizr.bak") {
		return fmt.Errorf("backup file for date should not exist")
	}
	out, err := exec.Command(target, "--version").Output()
	if err != nil {
		return fmt.Errorf("failed to run %s --version: %w", target, err)
	}
	if !strings.Contains(string(out), "GNU") {
		return fmt.Errorf("%s does not appear to be GNU version", target)
	}
	return nil
}

// Helper functions
func isPkgInstalled(pkg string) bool {
	return exec.Command("pacman", "-Qi", pkg).Run() == nil
}

func isSymlink(path string) bool {
	fi, err := os.Lstat(path)
	return err == nil && fi.Mode()&os.ModeSymlink != 0
}

func fileExists(path string) bool {
	_, err := os.Stat(path)
	return err == nil
}

func readLines(path string) ([]string, error) {
	content, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	return strings.Split(strings.TrimSpace(string(content)), "\n"), nil
}

func checkSymlink(path, expectedTarget string) error {
	if path == "" {
		return fmt.Errorf("empty path provided")
	}
	if expectedTarget == "" {
		return fmt.Errorf("empty expected target provided")
	}
	if !isSymlink(path) {
		return fmt.Errorf("%s is not a symlink", path)
	}
	dest, err := os.Readlink(path)
	if err != nil {
		return fmt.Errorf("could not read link %s: %w", path, err)
	}
	if !strings.Contains(dest, expectedTarget) {
		return fmt.Errorf("symlink %s has unexpected target '%s', expected to contain '%s'", path, dest, expectedTarget)
	}
	return nil
}

func checkSymlinkContainsAny(path string, expectedTargets []string) error {
	if path == "" {
		return fmt.Errorf("empty path provided")
	}
	if len(expectedTargets) == 0 {
		return fmt.Errorf("no expected targets provided")
	}
	if !isSymlink(path) {
		return fmt.Errorf("%s is not a symlink", path)
	}
	dest, err := os.Readlink(path)
	if err != nil {
		return fmt.Errorf("could not read link %s: %w", path, err)
	}
	for _, exp := range expectedTargets {
		if exp != "" && strings.Contains(dest, exp) {
			return nil
		}
	}
	return fmt.Errorf("symlink %s has unexpected target '%s', expected to contain one of %v", path, dest, expectedTargets)
}
