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

	content, err := os.ReadFile("/etc/os-release")
	if err != nil {
		return false, fmt.Errorf("could not read /etc/os-release: %w", err)
	}

	var currentDistro string
	for _, line := range strings.Split(string(content), "\n") {
		if strings.HasPrefix(line, "ID=") {
			currentDistro = strings.Trim(strings.Split(line, "=")[1], `"`)
			break
		}
	}

	if currentDistro == "" {
		return false, fmt.Errorf("could not determine distro from /etc/os-release")
	}

	for _, d := range distros {
		if strings.EqualFold(d, currentDistro) {
			return true, nil
		}
	}

	return false, nil
}
