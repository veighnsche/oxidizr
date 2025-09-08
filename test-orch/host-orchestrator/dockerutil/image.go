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
