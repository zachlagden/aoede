FROM rust:alpine AS dependencies

RUN apk add --no-cache alpine-sdk cmake automake autoconf opus libtool
# Fix for audiopus_sys crate using old CMakeLists.txt incompatible with newer CMake.
# Without this, the build fails with: "Compatibility with CMake < 3.5 has been removed from CMake"
ENV CMAKE_POLICY_VERSION_MINIMUM=3.5
RUN cargo install cargo-chef

FROM dependencies AS planner
WORKDIR app

# We only pay the installation cost once,
# it will be cached from the second build onwards
# To ensure a reproducible build consider pinning
# the cargo-chef version with `--version X.X.X`
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM dependencies AS cacher
WORKDIR app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM dependencies AS builder
WORKDIR app
COPY . .
# Copy over the cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo build --release --bin aoede

FROM alpine AS runtime
WORKDIR app
COPY --from=builder /app/target/release/aoede /usr/local/bin

ENV CACHE_DIR=/data

# Entrypoint script that applies the apresolve workaround at runtime.
# Docker replaces /etc/hosts at container start, so we must modify it then.
# Blocking apresolve.spotify.com forces librespot to use fallback access points
# (ap.spotify.com), which may help with connectivity on older librespot versions.
# See: https://github.com/librespot-org/librespot/issues/972
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
