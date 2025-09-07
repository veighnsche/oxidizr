# syntax=docker/dockerfile:1

# Builder stage for compiling the Go runner
ARG BASE_IMAGE=archlinux:base-devel
FROM golang:1.21 as builder

WORKDIR /build

# Copy the container runner source code
COPY container-runner/ ./

# Download dependencies
RUN go mod download

# Build the in-container runner
RUN CGO_ENABLED=0 GOOS=linux go build -o /isolated-runner .

# Final stage
FROM ${BASE_IMAGE}

ENV LANG=C.UTF-8

# Refresh system and install essentials used by our scripts
RUN pacman -Syu --noconfirm && \
    pacman -S --noconfirm reflector && \
    reflector --latest 10 --sort rate --save /etc/pacman.d/mirrorlist && \
    pacman -S --noconfirm --needed sudo git curl rustup which findutils glibc glibc-locales && \
    # Reduce image size a bit
    pacman -Scc --noconfirm || true

# Set DNS servers to fix network issues
RUN echo "nameserver 8.8.8.8" > /etc/resolv.conf && \
    echo "nameserver 8.8.4.4" >> /etc/resolv.conf

# Working directory where we will mount the repo
WORKDIR /workspace

# Copy the compiled Go runner from the builder stage, make it executable, and set it as the entrypoint.
COPY --from=builder /isolated-runner /usr/local/bin/isolated-runner
COPY docker/setup_shell.sh /usr/local/bin/setup_shell.sh
RUN chmod +x /usr/local/bin/isolated-runner && chmod +x /usr/local/bin/setup_shell.sh
ENTRYPOINT ["/usr/local/bin/isolated-runner"]
