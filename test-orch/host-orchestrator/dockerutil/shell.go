package dockerutil

import (
	"context"
	"fmt"
	"io"
	"log"
	"os"
	"path/filepath"
	"strings"
	"sync"

	"github.com/docker/docker/api/types"
	"github.com/docker/docker/api/types/container"
	"github.com/docker/docker/client"
	"github.com/moby/term"
)

// Verb, Allowed and Prefix are defined in util.go and shared across dockerutil.

// RunArchInteractiveShell starts an interactive shell inside the given image,
// wiring TTY/stdin/out and mounting the workspace and caches similarly to RunArchContainer.
// selected controls visibility of host-originated lines; distro is the raw name for prefixing.
func RunArchInteractiveShell(tag, rootDir string, selected Verb, distro string) error {
	if Allowed(selected, V2) {
		log.Printf("%s RUN> docker run -it -v %s:/workspace %s bash -l", Prefix(distro, V2, "HOST"), rootDir, tag)
	}
	ctx := context.Background()
	cli, err := client.NewClientWithOpts(client.FromEnv, client.WithAPIVersionNegotiation())
	if err != nil {
		return fmt.Errorf("docker client: %w", err)
	}
	exists, err := ImageExists(ctx, cli, tag)
	if err != nil {
		return err
	}
	if !exists {
		return fmt.Errorf("image %s not found", tag)
	}
	cfg := &container.Config{
		Image:      tag,
		Entrypoint: []string{"/bin/bash", "-c", "setup_shell.sh && bash -l"},
		Tty:        true,
		OpenStdin:  true,
	}
	// Add persistent cache mounts similar to non-interactive runs to avoid repeated downloads.
	// Derive a distro key from the tag, e.g., oxidizr-cachyos:latest -> cachyos
	distroKey := strings.TrimPrefix(tag, "oxidizr-")
	if i := strings.Index(distroKey, ":"); i >= 0 {
		distroKey = distroKey[:i]
	}
	cacheRoot := filepath.Join(rootDir, ".cache", "test-orch")
	// Namespace caches per-distro to avoid cross-container contention when running in parallel
	cargoReg := filepath.Join(cacheRoot, "cargo", "registry", distroKey)
	cargoGit := filepath.Join(cacheRoot, "cargo", "git", distroKey)
	cargoTarget := filepath.Join(cacheRoot, "cargo-target", distroKey)
	pacmanCache := filepath.Join(cacheRoot, "pacman", distroKey)
	aurBuild := filepath.Join(cacheRoot, "aur-build", distroKey)
	rustupRoot := filepath.Join(cacheRoot, "rustup", distroKey)
	for _, d := range []string{cargoReg, cargoGit, cargoTarget, pacmanCache, aurBuild, rustupRoot} {
		_ = os.MkdirAll(d, 0o755)
	}
	binds := []string{
		fmt.Sprintf("%s:/workspace", rootDir),
		fmt.Sprintf("%s:%s", cargoReg, "/root/.cargo/registry"),
		fmt.Sprintf("%s:%s", cargoGit, "/root/.cargo/git"),
		fmt.Sprintf("%s:%s", cargoTarget, "/workspace/target"),
		fmt.Sprintf("%s:%s", pacmanCache, "/var/cache/pacman"),
		fmt.Sprintf("%s:%s", aurBuild, "/home/builder/build"),
		fmt.Sprintf("%s:%s", rustupRoot, "/root/.rustup"),
	}
	hostCfg := &container.HostConfig{Binds: binds, AutoRemove: true}
	_ = cli.ContainerRemove(context.Background(), "oxidizr-arch-shell", types.ContainerRemoveOptions{Force: true})
	created, err := cli.ContainerCreate(ctx, cfg, hostCfg, nil, nil, "oxidizr-arch-shell")
	if err != nil {
		return fmt.Errorf("container create: %w", err)
	}
	inFd, _ := term.GetFdInfo(os.Stdin)
	_, isTerm := term.GetFdInfo(os.Stdout)
	var restore func() error
	if isTerm {
		state, err := term.MakeRaw(inFd)
		if err == nil {
			restore = func() error { return term.RestoreTerminal(inFd, state) }
		}
	}
	if restore != nil {
		defer restore()
	}
	attach, err := cli.ContainerAttach(ctx, created.ID, types.ContainerAttachOptions{
		Stream: true, Stdin: true, Stdout: true, Stderr: true, Logs: true,
	})
	if err != nil {
		return fmt.Errorf("container attach: %w", err)
	}
	defer attach.Close()
	if err := cli.ContainerStart(ctx, created.ID, types.ContainerStartOptions{}); err != nil {
		return fmt.Errorf("container start: %w", err)
	}
	var wg sync.WaitGroup
	wg.Add(2)
	go func() { defer wg.Done(); _, _ = io.Copy(attach.Conn, os.Stdin) }()
	go func() { defer wg.Done(); _, _ = io.Copy(os.Stdout, attach.Conn) }()

	// Wait for container to exit; treat normal exit as success.
	statusCh, errCh := cli.ContainerWait(ctx, created.ID, container.WaitConditionNotRunning)
	var exitCode int64 = 0
	select {
	case st := <-statusCh:
		exitCode = st.StatusCode
	case err := <-errCh:
		// Close streams and wait for copy goroutines to finish
		attach.Close()
		wg.Wait()
		return fmt.Errorf("container wait: %w", err)
	}

	// Close streams and wait for I/O goroutines to unwind cleanly
	attach.Close()
	wg.Wait()

	// Do not treat non-zero exit as an error in interactive mode; user may exit with custom code.
	if Allowed(selected, V2) {
		log.Printf("%s interactive shell exited with code %d", Prefix(distro, V2, "HOST"), exitCode)
	}
	return nil
}
