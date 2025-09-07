package main

import (
	"archive/tar"
	"context"
	"fmt"
	"io"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"time"

	"github.com/docker/docker/api/types"
	"github.com/docker/docker/api/types/container"
	"github.com/docker/docker/api/types/filters"
	"github.com/docker/docker/api/types/image"
	"github.com/docker/docker/api/types/strslice"
	"github.com/docker/docker/client"
	"github.com/docker/docker/pkg/jsonmessage"
	"github.com/moby/term"
)

// buildArchImage builds the Arch Docker image used for running the isolated tests.
func buildArchImage(tag, contextDir string, noCache, pull bool, verbose bool) error {
	if verbose {
		log.Println(prefixRun(), "docker build -t", tag, contextDir)
	}
	// Initialize Docker client
	ctx := context.Background()
	cli, err := client.NewClientWithOpts(client.FromEnv, client.WithAPIVersionNegotiation())
	if err != nil {
		return fmt.Errorf("docker client: %w", err)
	}
	// Create tar build context from contextDir
	buildCtx, err := tarDirectory(contextDir)
	if err != nil {
		return fmt.Errorf("create build context: %w", err)
	}
	defer buildCtx.Close()
	
	opts := types.ImageBuildOptions{
		Tags:       []string{tag},
		Remove:     true,
		NoCache:    noCache,
		PullParent: pull,
	}
	resp, err := cli.ImageBuild(ctx, buildCtx, opts)
	if err != nil {
		return fmt.Errorf("image build: %w", err)
	}
	defer resp.Body.Close()
	if verbose {
		// Render the daemon's JSON message stream in a human-friendly way
		fd, isTerm := term.GetFdInfo(os.Stdout)
		if err := jsonmessage.DisplayJSONMessagesStream(resp.Body, os.Stdout, fd, isTerm, nil); err != nil {
			return fmt.Errorf("render build output: %w", err)
		}
	} else {
		io.Copy(io.Discard, resp.Body)
	}
	return nil
}

// runArchInteractiveCommand starts an interactive TTY container and executes the provided
// command string (interpreted by the image entrypoint /bin/bash -lc). StdIO is attached.
func runArchInteractiveCommand(tag, rootDir, cmd string, verbose bool) error {
    if verbose {
        log.Println(prefixRun(), "docker run -it -v", rootDir+":/workspace", tag, cmd)
    }
    ctx := context.Background()
    cli, err := client.NewClientWithOpts(client.FromEnv, client.WithAPIVersionNegotiation())
    if err != nil {
        return fmt.Errorf("docker client: %w", err)
    }

    exists, err := imageExists(ctx, cli, tag)
    if err != nil {
        return err
    }
    if !exists {
        return fmt.Errorf("image %s not found (build it first with --arch or --arch-build)", tag)
    }

    cfg := &container.Config{
        Image:        tag,
        // Image ENTRYPOINT is ["/bin/bash","-lc"], so pass the single string command.
        Cmd:          strslice.StrSlice([]string{cmd}),
        Tty:          true,
        OpenStdin:    true,
        AttachStdout: true,
        AttachStderr: true,
        AttachStdin:  true,
    }
    hostCfg := &container.HostConfig{Binds: []string{fmt.Sprintf("%s:/workspace", rootDir)}, AutoRemove: true}
    // Best-effort remove any stale container with the same name to avoid name conflicts
    _ = cli.ContainerRemove(context.Background(), "oxidizr-arch-shell", container.RemoveOptions{Force: true})

    created, err := cli.ContainerCreate(ctx, cfg, hostCfg, nil, nil, "oxidizr-arch-shell")
    if err != nil {
        return fmt.Errorf("container create: %w", err)
    }

    // Put terminal in raw mode if stdout is a TTY
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

    // Attach to the container (request prior logs too, to capture prelude output)
    attach, err := cli.ContainerAttach(ctx, created.ID, container.AttachOptions{
        Stream: true, Stdin: true, Stdout: true, Stderr: true, Logs: true,
    })
    if err != nil {
        return fmt.Errorf("container attach: %w", err)
    }
    defer attach.Close()

    if err := cli.ContainerStart(ctx, created.ID, container.StartOptions{}); err != nil {
        return fmt.Errorf("container start: %w", err)
    }

    // Copy stdio (since TTY is true, the stream is raw)
    go func() { io.Copy(attach.Conn, os.Stdin) }()
    _, _ = io.Copy(os.Stdout, attach.Conn)

    return nil
}

// runArchInteractiveShell starts an interactive TTY container with the repo mounted at /workspace
// and attaches the current stdio to a bash login shell.
func runArchInteractiveShell(tag, rootDir string, verbose bool) error {
    if verbose {
        log.Println(prefixRun(), "docker run -it -v", rootDir+":/workspace", tag, "bash /workspace/test-orch/docker/prepare_and_shell.sh")
    }
    ctx := context.Background()
    cli, err := client.NewClientWithOpts(client.FromEnv, client.WithAPIVersionNegotiation())
    if err != nil {
        return fmt.Errorf("docker client: %w", err)
    }

	exists, err := imageExists(ctx, cli, tag)
	if err != nil {
		return err
	}
	if !exists {
		return fmt.Errorf("image %s not found (build it first with --arch or --arch-build)", tag)
	}

    cfg := &container.Config{
        Image:        tag,
        // The image ENTRYPOINT is ["/bin/bash","-lc"]. Pass a single string so it becomes the -c command.
        Cmd:          strslice.StrSlice([]string{"bash /workspace/test-orch/docker/prepare_and_shell.sh"}),
        Tty:          true,
        OpenStdin:    true,
        AttachStdout: true,
        AttachStderr: true,
        AttachStdin:  true,
    }
	hostCfg := &container.HostConfig{Binds: []string{fmt.Sprintf("%s:/workspace", rootDir)}, AutoRemove: true}
    // Best-effort remove any stale container with the same name to avoid name conflicts
    _ = cli.ContainerRemove(context.Background(), "oxidizr-arch-shell", container.RemoveOptions{Force: true})

    created, err := cli.ContainerCreate(ctx, cfg, hostCfg, nil, nil, "oxidizr-arch-shell")
	if err != nil {
		return fmt.Errorf("container create: %w", err)
	}

	// Put terminal in raw mode if stdout is a TTY
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

    // Attach to the container (request prior logs too, to capture prelude output)
    attach, err := cli.ContainerAttach(ctx, created.ID, container.AttachOptions{
        Stream: true, Stdin: true, Stdout: true, Stderr: true, Logs: true,
    })
    if err != nil {
        return fmt.Errorf("container attach: %w", err)
    }
    defer attach.Close()

    if err := cli.ContainerStart(ctx, created.ID, container.StartOptions{}); err != nil {
        return fmt.Errorf("container start: %w", err)
    }

    // Copy stdio (since TTY is true, the stream is raw)
    go func() { io.Copy(attach.Conn, os.Stdin) }()
    _, _ = io.Copy(os.Stdout, attach.Conn)

	return nil
}

// runArchContainer runs the Arch image with the repo mounted at /workspace and executes entrypoint.sh
func runArchContainer(tag, rootDir, entrypoint string, keepContainer bool, timeout time.Duration, verbose bool) error {
    containerName := "oxidizr-arch-test"
    if verbose {
        log.Println(prefixRun(), "docker run -v", rootDir+":/workspace", "--name", containerName, tag, entrypoint)
    }
    ctx, cancel := context.WithTimeout(context.Background(), timeout)
    defer cancel()

    // Always use Docker CLI for consistency. Stream output in verbose mode; discard in quiet.
    // Best-effort remove any stale container with the same name to avoid conflicts
    _ = exec.Command("docker", "rm", "-f", containerName).Run()
    args := []string{"run"}
    if !keepContainer { args = append(args, "--rm") }
    args = append(args,
        "-v", fmt.Sprintf("%s:/workspace", rootDir),
        "--workdir", "/workspace",
        "--name", containerName,
        tag,
        entrypoint,
    )
    cmd := exec.CommandContext(ctx, "docker", args...)
    if verbose {
        cmd.Stdout = os.Stdout
        cmd.Stderr = os.Stderr
    } else {
        cmd.Stdout = io.Discard
        cmd.Stderr = io.Discard
    }
    if err := cmd.Run(); err != nil {
        return fmt.Errorf("docker run (CLI) failed: %w", err)
    }
    return nil
}

// imageExists checks if a Docker image tag exists locally via SDK
func imageExists(ctx context.Context, cli *client.Client, tag string) (bool, error) {
	f := filters.NewArgs()
	f.Add("reference", tag)
	imgs, err := cli.ImageList(ctx, image.ListOptions{Filters: f})
	if err != nil {
		return false, fmt.Errorf("image list: %w", err)
	}
	return len(imgs) > 0, nil
}

// Tar the given directory into an io.ReadCloser suitable for Docker build context
func tarDirectory(dir string) (io.ReadCloser, error) {
	pr, pw := io.Pipe()
	tw := tar.NewWriter(pw)

	go func() {
		defer pw.Close()
		defer tw.Close()
		filepath.Walk(dir, func(path string, info os.FileInfo, err error) error {
			if err != nil {
				return err
			}
			// Docker build context expects paths relative to the context root
			rel, err := filepath.Rel(dir, path)
			if err != nil {
				return err
			}
			// Normalize directory headers
			hdr, err := tar.FileInfoHeader(info, "")
			if err != nil {
				return err
			}
			hdr.Name = rel
			if info.IsDir() {
				hdr.Name += "/"
			}
			if err := tw.WriteHeader(hdr); err != nil {
				return err
			}
			if info.Mode().IsRegular() {
				f, err := os.Open(path)
				if err != nil {
					return err
				}
				_, err = io.Copy(tw, f)
				f.Close()
				if err != nil {
					return err
				}
			}
			return nil
		})
	}()

	return pr, nil
}

// detectRepoRoot finds the git repository root, or returns an error.
func detectRepoRoot() (string, error) {
	// Prefer `git rev-parse --show-toplevel`
	out := out("git", "rev-parse", "--show-toplevel")
	if out != "" {
		return out, nil
	}
	// Fallback: search upwards for a Cargo.toml or .git directory as heuristic
	wd, err := os.Getwd()
	if err != nil {
		return "", err
	}
	dir := wd
	for i := 0; i < 6; i++ { // don't traverse indefinitely
		if _, err := os.Stat(filepath.Join(dir, ".git")); err == nil {
			return dir, nil
		}
		if _, err := os.Stat(filepath.Join(dir, "Cargo.toml")); err == nil {
			return dir, nil
		}
		parent := filepath.Dir(dir)
		if parent == dir {
			break
		}
		dir = parent
	}
	return "", fmt.Errorf("could not detect repo root")
}
