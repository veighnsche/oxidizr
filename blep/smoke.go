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
		log.Println(dockerTroubleshootTips("pull"))
		return false
	}
	cmd := []string{"run", "--rm", "archlinux:base-devel",
		"bash", "-lc", "set -euo pipefail; pacman -Syy --noconfirm >/dev/null; printf 'nameserver 1.1.1.1\n' >/etc/resolv.conf; ping -c1 -W3 archlinux.org >/dev/null && echo OK"}
	if verbose {
		log.Println(prefixRun(), "docker "+strings.Join(cmd, " "))
	}
	if err := run("docker", cmd...); err != nil {
		warn("Docker Arch smoke test failed. Check network reachability and DNS. Error: ", err)
		log.Println(dockerTroubleshootTips("smoke"))
		return false
	}
	log.Println("Docker Arch smoke test: OK")
	return true
}
