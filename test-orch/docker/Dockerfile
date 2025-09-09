# syntax=docker/dockerfile:1

ARG BASE_IMAGE=archlinux:base-devel
FROM ${BASE_IMAGE}

# Ensure locale files aren't stripped by pacman (some minimal images set NoExtract)
RUN sed -i 's/^[[:space:]]*NoExtract/# NoExtract (disabled for locales)/' /etc/pacman.conf

# Refresh system and install essentials used by the runner
RUN pacman -Syyu --noconfirm && \
    pacman -S --noconfirm reflector && \
    reflector --protocol https --latest 10 --sort rate --save /etc/pacman.d/mirrorlist && \
    pacman -S --noconfirm --needed \
        sudo git curl which findutils sed glibc \
        rustup \
        python python-yaml jq \
        base-devel tar gzip && \
    # Reduce image size a bit
    pacman -Scc --noconfirm || true

# Force reinstall glibc to restore any locale files that may have been stripped previously
RUN pacman -S --noconfirm --overwrite='*' glibc

# Enable and generate en_US.UTF-8 (default) and de_DE.UTF-8 (for tests)
RUN sed -i \
        -e 's/^[#[:space:]]*en_US.UTF-8 UTF-8/en_US.UTF-8 UTF-8/' \
        -e 's/^[#[:space:]]*de_DE.UTF-8 UTF-8/de_DE.UTF-8 UTF-8/' \
        /etc/locale.gen && \
    locale-gen

# Persist default locale (English) for non-login shells and child processes
RUN printf 'LANG=en_US.UTF-8\nLC_ALL=en_US.UTF-8\n' > /etc/locale.conf

# Now set environment defaults (after generation) to English
ENV LANG=en_US.UTF-8 \
    LC_ALL=en_US.UTF-8

# Pre-install Rust stable toolchain (no network at runtime)
RUN rustup toolchain install stable -y && \
    rustup default stable

# Set DNS servers to fix network issues
RUN echo "nameserver 8.8.8.8" > /etc/resolv.conf && \
    echo "nameserver 8.8.4.4" >> /etc/resolv.conf

# Working directory where we will mount the repo
WORKDIR /workspace

# Default to the Python runner inside the mounted repository
ENTRYPOINT ["python3", "/workspace/test-orch/container-runner/runner.py", "all"]
