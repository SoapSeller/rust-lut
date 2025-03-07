
FROM mcr.microsoft.com/devcontainers/rust:1

# Install Vulkan drivers
RUN sudo apt update && \
    sudo apt install -yq --no-install-recommends \
        mesa-vulkan-drivers \
        ocl-icd-opencl-dev \
        intel-opencl-icd \
        llvm-spirv-15 \
        clang-15 \
    && \
    sudo apt autoclean && \
    sudo apt -y autoremove && \
    sudo rm -rf /var/lib/apt/lists/*
