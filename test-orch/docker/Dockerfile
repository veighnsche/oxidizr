# syntax=docker/dockerfile:1
FROM archlinux:base-devel

ENV LANG=C.UTF-8
SHELL ["/bin/bash", "-lc"]

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

# Default entrypoint will be provided via run.sh (bind mounts scripts/ and repo)
ENTRYPOINT ["/bin/bash", "-lc"]
CMD ["echo", "Use run.sh to start the test runner with proper mounts."]
