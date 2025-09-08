package dockerutil

import (
	"archive/tar"
	"context"
	"fmt"
	"io"
	"log"
	"os"
	"path/filepath"

	"github.com/docker/docker/api/types"
	"github.com/docker/docker/client"
	"github.com/docker/docker/pkg/jsonmessage"
	"github.com/fatih/color"
	"github.com/moby/term"
)

func BuildArchImage(tag, contextDir, baseImage string, noCache, pull bool, verbose bool, prefix string, col *color.Color) error {
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
	// Always parse the JSON message stream so we can detect build errors even when not verbose.
	fd, isTerm := term.GetFdInfo(os.Stdout)
	var out io.Writer = io.Discard
	if verbose {
		// Use a custom writer to prefix output lines when verbose
		out = &prefixWriter{prefix: prefix, w: os.Stdout, col: col}
	}
	if err := jsonmessage.DisplayJSONMessagesStream(resp.Body, out, fd, isTerm, nil); err != nil {
		return fmt.Errorf("render build output: %w", err)
	}
	return nil
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
