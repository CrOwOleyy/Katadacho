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

# AppImage : linuxdeploy est lui-même un AppImage. Dans un conteneur, on le lance en mode extraction
# (pas de FUSE nécessaire), et il lui faut squashfs-tools pour écrire l'image finale.
RUN dnf install -y fuse fuse-libs squashfs-tools patchelf && dnf clean all
ENV APPIMAGE_EXTRACT_AND_RUN=1
