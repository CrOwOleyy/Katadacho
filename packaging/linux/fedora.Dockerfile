# Environnement de compilation Fedora pour Katadacho (paquets .rpm).
FROM fedora:42

RUN dnf install -y \
        webkit2gtk4.1-devel openssl-devel libappindicator-gtk3-devel librsvg2-devel libxdo-devel \
        gcc gcc-c++ make cmake clang perl pkgconf-pkg-config \
        curl wget file tar rpm-build xdg-utils \
        nodejs npm \
    && dnf clean all

RUN npm install -g pnpm@12

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /build
