package assertions

import (
	"fmt"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"

	"container-runner/util"
)

// Run executes the full assertion flow.
func Run() error {
	log.Println("==> Running assertion flow...")

	// Run on Arch-family by default; orchestrator limits runs to supported distros.
	archFamily := []string{"arch", "manjaro", "cachyos", "endeavourOS", "endeavouros", "endeavoros"}
	if ok, err := util.ShouldRunOnDistro(archFamily); err != nil {
		log.Printf("Skipping assertions: distro detection failed: %v. Before introducing any masking or skip logic, read WHY_LLMS_ARE_STUPID.md to avoid false positives.", err)
		return nil
	} else if !ok {
		log.Println("Skipping assertions on non-Arch-family distro. If tempted to bypass this check, read WHY_LLMS_ARE_STUPID.md to avoid programming in false positives.")
		return nil
	}

	// Enable experiments
	log.Println("--- Enabling experiments: coreutils,sudo-rs ---")
	if err := util.RunCmd("oxidizr-arch", "--assume-yes", "--experiments", "coreutils,sudo-rs", "--package-manager", "none", "enable"); err != nil {
		log.Printf("❌ Enable experiments (coreutils,sudo-rs): %v", err)
		return fmt.Errorf("failed to enable experiments: %w", err)
	}
	log.Println("✅ Enable experiments: coreutils,sudo-rs")

	// Assertions after enabling
	log.Println("--- Asserting sudo-rs is installed ---")
	if err := ensureSudoRsInstalled(); err != nil {
		log.Printf("❌ sudo-rs installed: %v", err)
		return err
	}
	log.Println("✅ sudo-rs installed")
	log.Println("--- Asserting coreutils is installed ---")
	if err := ensureCoreutilsInstalled(); err != nil {
		log.Printf("❌ coreutils installed: %v", err)
		return err
	}
	log.Println("✅ coreutils installed")

	// Disable experiments
	log.Println("--- Disabling experiments: coreutils,sudo-rs ---")
	if err := util.RunCmd("oxidizr-arch", "--assume-yes", "--experiments", "coreutils,sudo-rs", "--package-manager", "none", "disable"); err != nil {
		log.Printf("❌ Disable experiments (coreutils,sudo-rs): %v", err)
		return fmt.Errorf("failed to disable experiments: %w", err)
	}
	log.Println("✅ Disable experiments: coreutils,sudo-rs")

	// Assertions after disabling
	log.Println("--- Asserting sudo-rs is absent ---")
	if err := ensureSudoRsAbsent(); err != nil {
		log.Printf("❌ sudo-rs absent: %v", err)
		return err
	}
	log.Println("✅ sudo-rs absent")
	log.Println("--- Asserting coreutils is absent ---")
	if err := ensureCoreutilsAbsent(); err != nil {
		log.Printf("❌ coreutils absent: %v", err)
		return err
	}
	log.Println("✅ coreutils absent")

	log.Println("✅ All assertions passed.")
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
	projectDir := os.Getenv("PROJECT_DIR")
	if projectDir == "" {
		projectDir = "/workspace"
	}
	binsFile := filepath.Join(projectDir, "tests/lib/rust-coreutils-bins.txt")
	log.Printf("CTX> coreutils bins file: %s", binsFile)
	bins, err := readLines(binsFile)
	if err != nil {
		return fmt.Errorf("could not read coreutils bin list: %w", err)
	}
	// Minimum symlink coverage threshold (default 10). Can override via COREUTILS_MIN_SYMLINKS env.
	minSymlinks := 10
	if v := os.Getenv("COREUTILS_MIN_SYMLINKS"); v != "" {
		log.Printf("CTX> COREUTILS_MIN_SYMLINKS env=%q", v)
		if n, err := strconv.Atoi(v); err == nil && n > 0 {
			minSymlinks = n
		}
	}

	symlinkCount := 0
	checkedCount := 0
	// Only these applets will be asked for --version (expected to return 0 and not contain GNU)
	criticalVersionCheck := map[string]bool{
		"date": true,
		"ls": true,
		"readlink": true,
		"stat": true,
		"rm": true,
		"cp": true,
		"ln": true,
		"mv": true,
		"cat": true,
		"echo": true,
	}
	for _, bin := range bins {
		target := "/usr/bin/" + bin
		checkedCount++
		if !isSymlink(target) {
			continue
		}
		symlinkCount++
		if !fileExists("/usr/bin/." + bin + ".oxidizr.bak") {
			return fmt.Errorf("backup file for %s not found", target)
		}
		// Only run --version on critical subset; some applets (e.g., false) exit non-zero by design
		if criticalVersionCheck[bin] {
			out, err := exec.Command(target, "--version").Output()
			if err != nil {
				return fmt.Errorf("failed to run %s --version: %w", target, err)
			}
			if strings.Contains(string(out), "GNU") {
				return fmt.Errorf("%s appears to be GNU version", target)
			}
		}
	}
	if symlinkCount == 0 {
		return fmt.Errorf("no coreutils applet symlinks detected; enable likely failed")
	}
	if symlinkCount < minSymlinks {
		return fmt.Errorf("insufficient coreutils coverage: have %d symlinks, require at least %d", symlinkCount, minSymlinks)
	}
	log.Printf("CTX> coreutils coverage detail: checked=%d symlinks=%d min=%d", checkedCount, symlinkCount, minSymlinks)
	log.Printf("Coreutils coverage: %d/%d symlinks (min %d)", symlinkCount, checkedCount, minSymlinks)
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
