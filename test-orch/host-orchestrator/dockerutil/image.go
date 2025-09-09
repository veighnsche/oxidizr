package dockerutil

import (
	"context"
	"fmt"

	"github.com/docker/docker/api/types"
	"github.com/docker/docker/api/types/filters"
	"github.com/docker/docker/client"
)

// ImageExists checks if a Docker image with the given tag exists locally.
func ImageExists(ctx context.Context, cli *client.Client, tag string) (bool, error) {
	f := filters.NewArgs()
	f.Add("reference", tag)
	imgs, err := cli.ImageList(ctx, types.ImageListOptions{Filters: f})
	if err != nil {
		return false, fmt.Errorf("image list: %w", err)
	}
	return len(imgs) > 0, nil
}

// ImageDigest returns the image ID/digest for the given tag, e.g. "sha256:...".
// If the image is not found locally, an error is returned.
func ImageDigest(ctx context.Context, cli *client.Client, tag string) (string, error) {
	inspect, _, err := cli.ImageInspectWithRaw(ctx, tag)
	if err != nil {
		return "", fmt.Errorf("image inspect: %w", err)
	}
	// Prefer RepoDigests when present; fall back to ID
	if len(inspect.RepoDigests) > 0 {
		return inspect.RepoDigests[0], nil
	}
	return inspect.ID, nil
}
