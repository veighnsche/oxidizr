package dockerutil

import (
	"fmt"
	"log"
	"os"
	"path/filepath"
	"strings"

	"github.com/fatih/color"
)

// RunOptions encapsulates configuration to construct `docker run` args.
type RunOptions struct {
	Tag          string
	RootDir      string
	Command      string
	EnvVars      []string
	KeepContainer bool
	Selected     Verb
	Distro       string
	Col          *color.Color
}

// BuildDockerRunArgs assembles the argument list for `docker run`, ensures
// cache directories exist, and returns the computed container name and logs dir.
func BuildDockerRunArgs(opts RunOptions) (args []string, containerName string, logsDir string) {
	containerName = fmt.Sprintf("oxidizr-arch-test-%s", strings.ReplaceAll(opts.Tag, ":", "-"))

	args = []string{"run"}
	if !opts.KeepContainer {
		args = append(args, "--rm")
	}
	for _, env := range opts.EnvVars {
		args = append(args, "-e", env)
	}

	// Provide distro identifier to in-container runner for analytics/report naming
	distroKey := strings.TrimPrefix(opts.Tag, "oxidizr-")
	if i := strings.Index(distroKey, ":"); i >= 0 {
		distroKey = distroKey[:i]
	}
	args = append(args, "-e", fmt.Sprintf("ANALYTICS_DISTRO=%s", distroKey))
	args = append(args, "-v", fmt.Sprintf("%s:/workspace", opts.RootDir))

	// Optional: override baked runner with host-built binary for fast iteration
	// When RUNNER_FROM_WORKSPACE=1 and the binary exists at test-orch/container-runner/isolated-runner,
	// bind-mount it to /usr/local/bin/isolated-runner inside the container.
	if os.Getenv("RUNNER_FROM_WORKSPACE") == "1" {
		hostRunner := filepath.Join(opts.RootDir, "test-orch", "container-runner", "isolated-runner")
		if fi, err := os.Stat(hostRunner); err == nil && !fi.IsDir() {
			args = append(args, "-v", fmt.Sprintf("%s:%s", hostRunner, "/usr/local/bin/isolated-runner:ro"))
			if Allowed(opts.Selected, V2) {
				log.Printf("%s CTX> overriding runner with host binary: %s -> /usr/local/bin/isolated-runner", opts.Col.Sprint(Prefix(opts.Distro, V2, "HOST")), hostRunner)
			}
		} else if Allowed(opts.Selected, V2) {
			log.Printf("%s WARN> RUNNER_FROM_WORKSPACE=1 set, but host runner not found at %s; using image-baked runner", opts.Col.Sprint(Prefix(opts.Distro, V2, "HOST")), hostRunner)
		}
	}

	// Add persistent cache mounts to speed up repeated runs
	cacheRoot := filepath.Join(opts.RootDir, ".cache", "test-orch")
	if i := strings.Index(distroKey, ":"); i >= 0 {
		distroKey = distroKey[:i]
	}
	// Namespace caches per-distro to avoid cross-container contention
	cargoReg := filepath.Join(cacheRoot, "cargo", "registry", distroKey)
	cargoGit := filepath.Join(cacheRoot, "cargo", "git", distroKey)
	cargoTarget := filepath.Join(cacheRoot, "cargo-target", distroKey)
	pacmanCache := filepath.Join(cacheRoot, "pacman", distroKey)
	// Make AUR build cache per-distro to avoid concurrent access and cross-distro conflicts
	aurBuild := filepath.Join(cacheRoot, "aur-build", distroKey)
	rustupRoot := filepath.Join(cacheRoot, "rustup", distroKey)
	// Ensure directories exist (including top-level logs dir for container logs)
	logsDir = filepath.Join(opts.RootDir, "logs", distroKey)
	for _, d := range []string{cargoReg, cargoGit, cargoTarget, pacmanCache, aurBuild, rustupRoot, logsDir} {
		_ = os.MkdirAll(d, 0o755)
	}
	// Bind mounts
	args = append(args, "-v", fmt.Sprintf("%s:%s", cargoReg, "/root/.cargo/registry"))
	args = append(args, "-v", fmt.Sprintf("%s:%s", cargoGit, "/root/.cargo/git"))
	args = append(args, "-v", fmt.Sprintf("%s:%s", cargoTarget, "/workspace/target"))
	args = append(args, "-v", fmt.Sprintf("%s:%s", pacmanCache, "/var/cache/pacman"))
	args = append(args, "-v", fmt.Sprintf("%s:%s", aurBuild, "/home/builder/build"))
	args = append(args, "-v", fmt.Sprintf("%s:%s", rustupRoot, "/root/.rustup"))
	args = append(args, "--workdir", "/workspace")
	args = append(args, "--name", containerName)
	args = append(args, opts.Tag)
	if opts.Command != "" {
		args = append(args, opts.Command)
	}
	return args, containerName, logsDir
}
