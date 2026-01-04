FROM rust:1.92-slim

# Common native deps for popular Rust crates (openssl, clang/bindgen).
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        pkg-config \
        libssl-dev \
        clang \
        curl \
        gosu \
        # nannou / winit runtime deps
        libx11-dev \
        libxi-dev \
        libgl1-mesa-dev \
        libasound2-dev \
        libxrandr-dev \
        libxcursor-dev \
        libxinerama-dev \
        libxkbcommon-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

# Install useful Rust components up front.
RUN rustup component add clippy rustfmt

# Match container user to host so mounted files are writable.
ARG USER_ID=1000
ARG GROUP_ID=1000
RUN groupadd -g "${GROUP_ID}" app \
    && useradd -l -u "${USER_ID}" -g app -m app \
    && chown -R app:app /workspace

# Copy and configure entrypoint script
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

ENV CARGO_TARGET_DIR=/workspace/target

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
# Default to an interactive shell; override with `docker compose run`.
CMD ["bash"]
