#!/usr/bin/env bash
# Setup required to build oxdraw

set -eu
script_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )
cd "$script_path/.."
set -x

# Prepare for installation
if [ -x "$(command -v apt-get)" ]; then
    sudo apt-get update
elif [ -x "$(command -v dnf)" ]; then
    sudo dnf check-update
fi


# Linux/Ubuntu required dependencies
# libxdo-dev required for menu see (https://github.com/tauri-apps/muda/tree/dev)
if [ -x "$(command -v apt-get)" ]; then
    sudo apt-get -y install \
        build-essential \
        curl \
        lld \
        clang \
        libgtk-3-dev \
        libxdo-dev \
        libssl-dev \
        pkg-config \
        libglib2.0-dev
elif [ -x "$(command -v dnf)" ]; then
    sudo dnf install \
        perl \
        lld \
        clang \
        clang-devel \
        clang-tools-extra \
        libxcb-devel \
        libxkbcommon-devel \
        openssl-devel \
        pkg-config
elif [ -x "$(command -v pacman)" ]; then
    sudo pacman -S \
        base-devel \
        lld \
        clang \
        xdotool
fi


# Install rust on the host machine
if [ -x "$(command -v curl)" ]; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
elif [ -x "$(command -v dnf)" ];   then
    sudo dnf install rust
fi

echo "setup.sh completed!"
