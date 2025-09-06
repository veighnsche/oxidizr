package main

import (
	"log"
)

// checkDocker validates Docker presence and daemon responsiveness.
func checkDocker(verbose bool) bool {
	section("Docker checks")
	ok := true
	if !have("docker") {
		warn("docker not found on PATH.")
		log.Println(dockerInstallHelp())
		return false
	}
	if verbose {
		log.Println(prefixRun(), "docker version --format '{{.Client.Version}}' (client)")
	}
	if err := runSilent("docker", "version"); err != nil {
		warn("docker is installed but not responding. Make sure the Docker daemon is running and your user is in the docker group.")
		log.Println("Quick fix:")
		log.Println("  sudo systemctl enable --now docker")
		log.Println("  sudo usermod -aG docker \"$USER\"  # then re-login or run: newgrp docker")
		log.Println(dockerTroubleshootTips("daemon"))
		ok = false
	}
	return ok
}

// dockerInstallHelp returns Debian/Ubuntu and Arch/Manjaro install guidance.
func dockerInstallHelp() string {
	return `
Docker does not seem to be installed.

Install Docker (Debian/Ubuntu and Arch):

Ubuntu / Debian
  sudo apt-get update
  sudo apt-get install -y ca-certificates curl gnupg
  sudo install -m 0755 -d /etc/apt/keyrings
  curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
  echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
  sudo apt-get update
  sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

Arch Linux / Manjaro
  sudo pacman -Syu --noconfirm
  sudo pacman -S --noconfirm docker docker-compose

Post-install (all distros)
  sudo systemctl enable --now docker
  sudo usermod -aG docker "$USER"   # re-login or: newgrp docker

Verify
  docker --version && docker compose version
  docker run --rm hello-world

Once Docker is installed, you can retry:
  go run . --smoke-arch-docker
`
}

// dockerTroubleshootTips provides additional, context-aware guidance for common Docker issues.
// The ctx parameter can be one of: "daemon", "pull", "smoke", "build", "run".
func dockerTroubleshootTips(ctx string) string {
	header := "\nAdditional troubleshooting tips:\n\n"
	common := `
General
  - Check the Docker service: sudo systemctl status docker
  - View recent logs:       journalctl -u docker --no-pager -n 200
  - Verify permissions:     id -nG | grep -qw docker  # if not, add and re-login
  - Socket permissions:     ls -l /var/run/docker.sock
  - Disk space:             docker system df ; df -h
  - Clean up:               docker system prune -f

`

	daemon := `
Daemon not responding
  - Start/enable service:   sudo systemctl enable --now docker
  - Add user to group:      sudo usermod -aG docker "$USER" && newgrp docker
  - After upgrades, reboot the host if the daemon fails to start cleanly.

`

	network := `
Network / Pull issues
  - DNS: Try public DNS. Edit /etc/docker/daemon.json and add:
      { "dns": ["8.8.8.8", "1.1.1.1"] }
    Then restart: sudo systemctl restart docker
  - Corporate proxy: export HTTP_PROXY/HTTPS_PROXY and NO_PROXY="localhost,127.0.0.1,::1,registry-1.docker.io"
  - Test connectivity: ping -c1 registry-1.docker.io ; curl -I https://registry-1.docker.io
  - Check firewall or VPN settings that may block container network.

`

	build := `
Build issues
  - Ensure Dockerfile context path is correct and accessible.
  - Try without cache:      docker build --no-cache -t <tag> <context>
  - If hitting permission denied on context: verify file ownership and FS permissions.

`

	run := `
Run issues
  - Validate bind mounts:   ensure the host path exists and you have read/write perms.
  - Timeouts:               increase --timeout flag; check heavy network operations.
  - Stuck containers:       docker ps -a ; docker rm -f <stuck>

`

	tips := header + common
	switch ctx {
	case "daemon":
		tips += daemon
	case "pull", "smoke":
		tips += network
	case "build":
		tips += build
	case "run":
		tips += run
	default:
		// no extra
	}
	return tips
}
