package main

import (
	"log"
	"strings"
)

func smokeTestDockerArch(verbose bool) bool {
	section("Docker Arch smoke test")
	if !have("docker") {
		warn("docker missing; cannot run smoke test")
		return false
	}
	// Pull a minimal image and run quick commands
	if verbose {
		log.Println(prefixRun(), "docker pull archlinux:base-devel")
	}
	if err := run("docker", "pull", "archlinux:base-devel"); err != nil {
		warn("failed to pull archlinux:base-devel: ", err)
		if verbose {
			log.Println(dockerTroubleshootTips("pull"))
		} else {
			log.Println("Hint: check network/proxy/DNS and Docker daemon status. Run with -v for detailed tips.")
		}
		return false
	}
	cmd := []string{"run", "--rm", "archlinux:base-devel",
		"bash", "-lc", "set -euo pipefail; pacman -Syy --noconfirm >/dev/null; printf 'nameserver 1.1.1.1\n' >/etc/resolv.conf; ping -c1 -W3 archlinux.org >/dev/null && echo OK"}
	if verbose {
		log.Println(prefixRun(), "docker "+strings.Join(cmd, " "))
	}
	if err := run("docker", cmd...); err != nil {
		warn("Docker Arch smoke test failed. Check network reachability and DNS. Error: ", err)
		if verbose {
			log.Println(dockerTroubleshootTips("smoke"))
		} else {
			log.Println("Hint: verify DNS (e.g., set resolv.conf) and connectivity. Run with -v for detailed tips.")
		}
		return false
	}
	log.Println("Docker Arch smoke test: OK")
	return true
}
