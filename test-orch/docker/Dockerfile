# syntax=docker/dockerfile:1

# Builder stage for compiling the Go runner
FROM golang:1.21 as builder

WORKDIR /build

# Copy the container runner source code
COPY container-runner/ ./

# Download dependencies
RUN go mod download

# Build the in-container runner
RUN CGO_ENABLED=0 GOOS=linux go build -o /isolated-runner .

# Final stage
FROM archlinux:base-devel


ENV LANG=C.UTF-8

# Refresh system and install essentials used by our scripts
RUN pacman -Syu --noconfirm && \
    pacman -S --noconfirm --needed sudo git curl rustup which findutils \
        uutils-coreutils sudo-rs && \
    # Create non-root builder user for building AUR packages
    useradd -m builder && \
    install -d -m 0755 -o root -g root /etc/sudoers.d && \
    printf 'builder ALL=(ALL) NOPASSWD: ALL\n' > /etc/sudoers.d/99-builder && chmod 0440 /etc/sudoers.d/99-builder && \
    # Pre-seed rustup for root and builder (some AUR prepare() steps require cargo)
    rustup default stable || true && \
    su - builder -c 'rustup default stable || true' && \
    # Reduce image size a bit
    pacman -Scc --noconfirm || true

# Working directory where we will mount the repo
WORKDIR /workspace

# Copy the compiled Go runner from the builder stage, make it executable, and set it as the entrypoint.
COPY --from=builder /isolated-runner /usr/local/bin/isolated-runner
RUN chmod +x /usr/local/bin/isolated-runner
ENTRYPOINT ["/usr/local/bin/isolated-runner"]
