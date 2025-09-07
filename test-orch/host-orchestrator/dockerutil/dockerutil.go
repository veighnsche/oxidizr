package dockerutil

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
	"github.com/docker/docker/client"
	"github.com/docker/docker/pkg/jsonmessage"
	"github.com/moby/term"
)

func BuildArchImage(tag, contextDir, baseImage string, noCache, pull bool, verbose bool) error {
	if baseImage == "" {
		baseImage = "archlinux:base-devel"
	}
	if verbose {
		log.Println("RUN>", "docker build -t", tag, contextDir)
	}
	ctx := context.Background()
	cli, err := client.NewClientWithOpts(client.FromEnv, client.WithAPIVersionNegotiation())
	if err != nil {
		return fmt.Errorf("docker client: %w", err)
	}
	buildCtx, err := TarDirectory(contextDir)
	if err != nil {
		return fmt.Errorf("create build context: %w", err)
	}
	defer buildCtx.Close()
	opts := types.ImageBuildOptions{
		Tags:       []string{tag},
		Remove:     true,
		NoCache:    noCache,
		PullParent: pull,
		Dockerfile: "docker/Dockerfile",
		BuildArgs: map[string]*string{
			"BASE_IMAGE": &baseImage,
		},
	}
	resp, err := cli.ImageBuild(ctx, buildCtx, opts)
	if err != nil {
		return fmt.Errorf("image build: %w", err)
	}
	defer resp.Body.Close()
	if verbose {
		fd, isTerm := term.GetFdInfo(os.Stdout)
		if err := jsonmessage.DisplayJSONMessagesStream(resp.Body, os.Stdout, fd, isTerm, nil); err != nil {
			return fmt.Errorf("render build output: %w", err)
		}
	} else {
		io.Copy(io.Discard, resp.Body)
	}
	return nil
}

func RunArchInteractiveShell(tag, rootDir string, verbose bool) error {
	if verbose {
		log.Println("RUN>", "docker run -it -v", rootDir+":/workspace", tag, "bash -l")
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
	hostCfg := &container.HostConfig{Binds: []string{fmt.Sprintf("%s:/workspace", rootDir)}, AutoRemove: true}
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
	go func() { _, _ = io.Copy(attach.Conn, os.Stdin) }()
	_, _ = io.Copy(os.Stdout, attach.Conn)
	return nil
}

func RunArchContainer(tag, rootDir, command string, envVars []string, keepContainer bool, timeout time.Duration, verbose bool) error {
	containerName := "oxidizr-arch-test"
	if verbose {
		log.Println("RUN>", "docker run", "-v", rootDir+":/workspace", "--name", containerName, tag, command)
	}
	ctx, cancel := context.WithTimeout(context.Background(), timeout)
	defer cancel()

	_ = exec.Command("docker", "rm", "-f", containerName).Run()

	args := []string{"run"}
	if !keepContainer {
		args = append(args, "--rm")
	}
	for _, env := range envVars {
		args = append(args, "-e", env)
	}
	args = append(args, "-v", fmt.Sprintf("%s:/workspace", rootDir))
	args = append(args, "--workdir", "/workspace")
	args = append(args, "--name", containerName)
	args = append(args, tag)
	if command != "" {
		args = append(args, command)
	}

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

func ImageExists(ctx context.Context, cli *client.Client, tag string) (bool, error) {
	f := filters.NewArgs()
	f.Add("reference", tag)
	imgs, err := cli.ImageList(ctx, types.ImageListOptions{Filters: f})
	if err != nil {
		return false, fmt.Errorf("image list: %w", err)
	}
	return len(imgs) > 0, nil
}

func TarDirectory(dir string) (io.ReadCloser, error) {
	pr, pw := io.Pipe()
	tw := tar.NewWriter(pw)
	go func() {
		defer pw.Close()
		defer tw.Close()
		filepath.Walk(dir, func(path string, info os.FileInfo, err error) error {
			if err != nil {
				return err
			}
			rel, err := filepath.Rel(dir, path)
			if err != nil {
				return err
			}
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
				defer f.Close()
				_, err = io.Copy(tw, f)
				if err != nil {
					return err
				}
			}
			return nil
		})
	}()
	return pr, nil
}
