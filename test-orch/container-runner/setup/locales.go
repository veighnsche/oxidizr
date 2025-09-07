package setup

import (
	"fmt"
	"log"
	"os"

	"container-runner/util"
)

// ensureGermanLocaleData downloads the de_DE locale file into /usr/share/i18n/locales/de_DE
// when it is missing on stripped derivative images. Uses curl which is ensured in deps.
func ensureGermanLocaleData() error {
	if err := os.MkdirAll("/usr/share/i18n/locales", 0755); err != nil {
		return fmt.Errorf("failed to create locales dir: %w", err)
	}
	targets := []string{
		"https://raw.githubusercontent.com/bminor/glibc/master/localedata/locales/de_DE",
		"https://git.savannah.gnu.org/gitweb/?p=glibc.git;a=blob_plain;f=localedata/locales/de_DE",
	}
	for _, u := range targets {
		if err := util.RunCmd("sh", "-lc", fmt.Sprintf("curl -fsSL '%s' -o /usr/share/i18n/locales/de_DE", u)); err == nil {
			// Basic sanity check: file should be non-empty
			if st, err2 := os.Stat("/usr/share/i18n/locales/de_DE"); err2 == nil && st.Size() > 0 {
				log.Println("Fetched de_DE locale definition from:", u)
				return nil
			}
		}
	}
	return fmt.Errorf("could not fetch de_DE locale definition from known sources")
}

// setupLocales ensures commonly used locales (en_US.UTF-8, de_DE.UTF-8, C.UTF-8) are generated.
// If the German locale definition is missing on some derivative images, it attempts a best-effort
// remediation by reinstalling glibc-locales and preparing /etc/locale.gen.
func setupLocales() error {
	// Check if German locale definition files exist
	if _, err := os.Stat("/usr/share/i18n/locales/de_DE"); os.IsNotExist(err) {
		log.Println("[locales] de_DE definition missing; attempting remediation")

		// Try to reinstall glibc-locales to get complete locale data
		if err := util.RunCmd("pacman", "-S", "--noconfirm", "glibc-locales"); err != nil {
			log.Printf("[locales] glibc-locales reinstall failed: %v", err)
		}

		// If still missing, ensure locale.gen exists with standard entries
		if _, err := os.Stat("/usr/share/i18n/locales/de_DE"); os.IsNotExist(err) {
			// Attempt to fetch de_DE locale data from upstream glibc as a last resort.
			log.Println("[locales] still missing; downloading de_DE from upstream glibc...")
			if err := ensureGermanLocaleData(); err != nil {
				log.Printf("[locales] fetch de_DE failed: %v", err)
			}
		}
	}

	// Ensure locale.gen exists so locale-gen can attempt generation
	if err := ensureLocaleGen(); err != nil {
		return fmt.Errorf("failed to setup locale.gen: %w", err)
	}

	// Try directly generating de_DE via localedef as an additional fallback
	_ = util.RunCmd("localedef", "-v", "-c", "-i", "de_DE", "-f", "UTF-8", "de_DE.UTF-8")

	// Pre-generate common locales used by tests
	if err := generateCommonLocales(); err != nil {
		log.Printf("[locales] locale-gen failed: %v", err)
		// If FULL_MATRIX is requested, escalate to error so we fail fast here with clear logs
		if os.Getenv("FULL_MATRIX") == "1" {
			return fmt.Errorf("locale generation failed under FULL_MATRIX")
		}
	}

	// Validate presence in locale archive
	if err := util.RunCmd("sh", "-lc", `localedef --list-archive | grep -q '^de_DE\.UTF-8$'`); err != nil {
		log.Println("[locales] de_DE.UTF-8 not present in archive after generation attempts")
		if os.Getenv("FULL_MATRIX") == "1" {
			return fmt.Errorf("de_DE.UTF-8 not present after generation under FULL_MATRIX")
		}
	} else {
		log.Println("[locales] de_DE.UTF-8 present in archive")
	}

	return nil
}

func ensureLocaleGen() error {
	// Check if locale.gen exists and has content
	if stat, err := os.Stat("/etc/locale.gen"); err == nil && stat.Size() > 0 {
		return nil // File exists and has content
	}

	log.Println("Creating /etc/locale.gen with standard entries...")
	content := `# This file lists locales that you wish to have built. You can find a list
# of valid locales in /usr/share/i18n/locales/. Please consult the 
# locale.gen(5) manual page for more information.
#
# Format: Language[_Territory][.Codeset][@Modifier]
#
# Common locales used by tests
en_US.UTF-8 UTF-8
de_DE.UTF-8 UTF-8
C.UTF-8 UTF-8
`

	if err := os.WriteFile("/etc/locale.gen", []byte(content), 0644); err != nil {
		return fmt.Errorf("failed to write locale.gen: %w", err)
	}

	return nil
}

func generateCommonLocales() error {
	log.Println("Pre-generating common locales...")

	// Ensure locale.gen has the entries we need
	if err := ensureLocaleGen(); err != nil {
		return err
	}

	// Generate locales
	if err := util.RunCmd("locale-gen"); err != nil {
		return fmt.Errorf("locale generation failed: %w", err)
	}

	log.Println("Locales generated successfully")
	return nil
}
