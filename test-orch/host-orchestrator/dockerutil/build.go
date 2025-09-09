package dockerutil

import (
	"archive/tar"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"os"
	"path/filepath"
	"regexp"
	"strings"

	"github.com/docker/docker/api/types"
	"github.com/docker/docker/client"
	"github.com/docker/docker/pkg/jsonmessage"
	"github.com/fatih/color"
	"github.com/moby/term"
)

// BuildArchImage builds the Docker image for the in-container runner.
// selected controls visibility; stream lines are intrinsically v3 (trace).
// distro is the raw distro name (e.g., "arch").
func BuildArchImage(tag, contextDir, baseImage string, noCache, pull bool, selected Verb, distro string, col *color.Color) error {
	if baseImage == "" {
		baseImage = "archlinux:base-devel"
	}
	if Allowed(selected, V2) { // command echo at v2
		log.Printf("%s RUN> docker build -t %s %s", col.Sprint(Prefix(distro, V2, "HOST")), tag, contextDir)
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
	// Decode JSON message stream to emit:
	// - v1 summarized step lines (e.g., "Step X/Y : <instruction>")
	// - v3 full stream lines
	fd, isTerm := term.GetFdInfo(os.Stdout)
	_ = fd
	_ = isTerm
	var v3out io.Writer = io.Discard
	if Allowed(selected, V3) {
		v3out = &prefixWriter{distro: distro, lvl: V3, scope: "HOST", w: os.Stdout, col: col}
	}
	dec := json.NewDecoder(resp.Body)
	// For v1, instead of printing every step line, show a compact progress bar updated in-place.
	// Only do this when not also showing v3 output, to avoid noisy duplication.
	showV1Progress := Allowed(selected, V1) && !Allowed(selected, V3)
	stepRe := regexp.MustCompile(`^Step\s+(\d+)/(\d+)`)
	curStep, totalSteps := 0, 0
	progressShown := false
	finishProgress := func() {
		if progressShown {
			fmt.Println() // finalize the progress line
			progressShown = false
		}
	}
	for {
		var jm jsonmessage.JSONMessage
		if err := dec.Decode(&jm); err != nil {
			if err == io.EOF {
				break
			}
			finishProgress()
			return fmt.Errorf("decode build stream: %w", err)
		}
		if jm.Error != nil {
			// Echo error at v1 and return
			finishProgress()
			if Allowed(selected, V1) {
				log.Printf("%s build error: %s", Prefix(distro, V1, "HOST"), jm.Error.Message)
			}
			return fmt.Errorf("image build error: %s", jm.Error.Message)
		}
		// v3: write raw lines similarly to DisplayJSONMessagesStream behavior
		if jm.Stream != "" {
			for _, line := range strings.Split(jm.Stream, "\n") {
				if line == "" {
					continue
				}
				if v3out != io.Discard {
					fmt.Fprintln(v3out, line)
				}
				// v1: render progress bar from "Step X/Y :" lines (hide details)
				if showV1Progress {
					if m := stepRe.FindStringSubmatch(strings.TrimSpace(line)); len(m) == 3 {
						// Parse numbers
						var x, y int
						fmt.Sscanf(m[1], "%d", &x)
						fmt.Sscanf(m[2], "%d", &y)
						if x > 0 && y > 0 {
							curStep, totalSteps = x, y
							// Build bar
							width := 24
							filled := int(float64(width) * float64(curStep) / float64(totalSteps))
							if filled > width { filled = width }
							bar := strings.Repeat("=", filled) + strings.Repeat(" ", width-filled)
							// Print in-place
							fmt.Printf("\r%s Building image [%s] (%d/%d)", col.Sprint(Prefix(distro, V1, "HOST")), bar, curStep, totalSteps)
							progressShown = true
						} else if strings.HasPrefix(line, "Successfully ") {
							// Finish bar on success; do not print extra summary in v1 progress mode
							finishProgress()
						}
					}
				}
			}
			continue
		}
		// Status updates (layer pulls, cache info, etc.)
		if jm.Status != "" {
			// v1: if showing progress, suppress status chatter (keep bar-only)
			if Allowed(selected, V1) && !showV1Progress {
				if strings.HasPrefix(jm.Status, "Pulling from") || strings.HasPrefix(jm.Status, "Digest:") || strings.HasPrefix(jm.Status, "Status:") {
					log.Printf("%s %s", Prefix(distro, V1, "HOST"), jm.Status)
				}
			}
			if v3out != io.Discard {
				// Include ID when present for trace
				if jm.ID != "" {
					fmt.Fprintf(v3out, "%s %s\n", jm.ID, jm.Status)
				} else {
					fmt.Fprintf(v3out, "%s\n", jm.Status)
				}
			}
		}
	}
	finishProgress()
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
