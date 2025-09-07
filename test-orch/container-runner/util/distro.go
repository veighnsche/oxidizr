package util

import (
	"fmt"
	"os"
	"strings"
)

func ShouldRunOnDistro(distros []string) (bool, error) {
	if len(distros) == 0 {
		return true, nil // No check means compatible with all
	}

	currentDistro, err := CurrentDistroID()
	if err != nil {
		return false, err
	}

	for _, d := range distros {
		if strings.EqualFold(d, currentDistro) {
			return true, nil
		}
	}

	return false, nil
}

// CurrentDistroID returns the lowercase ID from /etc/os-release (e.g., "arch", "manjaro", "cachyos", "endeavouros").
func CurrentDistroID() (string, error) {
	content, err := os.ReadFile("/etc/os-release")
	if err != nil {
		return "", fmt.Errorf("could not read /etc/os-release: %w", err)
	}

	var id string
	for _, line := range strings.Split(string(content), "\n") {
		if strings.HasPrefix(line, "ID=") {
			id = strings.Trim(strings.Split(line, "=")[1], `"`)
			break
		}
	}
	if id == "" {
		return "", fmt.Errorf("could not determine distro from /etc/os-release")
	}
	return strings.ToLower(id), nil
}
