#!/bin/sh

SERVER_VERSION="0.1.3"
SERVER_SOURCE="https://github.com/paulusminus/pm"
SERVER_DESCRIPTION="implements http handler for managing lyrics and playlists"
SERVER_LICENSES="MIT"


# Get Dependencies
wkg oci pull -o deps/pm/lipl-service.wasm ghcr.io/paulusminus/pm/lipl-service:0.1.3
wkg oci pull -o deps/pm/lipl-storage-fs.wasm ghcr.io/paulusminus/pm/lipl-storage-fs:0.1.8

# Perform Composition
wac compose compose.wac -o out/server.wasm

# Push to Registry
wkg oci push \
  -a "Paul Min" \
  --annotation org.opencontainers.image.source="${SERVER_SOURCE}" \
  --annotation org.opencontainers.image.description="${SERVER_DESCRIPTION}" \
  --annotation org.opencontainers.image.version="${SERVER_VERSION}" \
  --annotation org.opencontainers.image.licenses="${SERVER_LICENSES}" \


  ghcr.io/paulusminus/pm/lipl-server:${SERVER_VERSION} \
  out/server.wasm
