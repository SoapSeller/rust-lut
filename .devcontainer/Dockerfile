
FROM mcr.microsoft.com/devcontainers/rust:1

# Install Vulkan drivers
RUN sudo apt update && \
    sudo apt install -yq --no-install-recommends \
    mesa-vulkan-drivers && \
    sudo apt autoclean && \
    sudo apt -y autoremove && \
    sudo rm -rf /var/lib/apt/lists/*
