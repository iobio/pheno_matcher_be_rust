# syntax=docker/dockerfile:1
ARG RUST_VERSION=latest
ARG APP_NAME=pheno_matcher_be_rust

# FROM --platform=$BUILDPLATFORM tonistiigi/xx:1.3.0 AS xx
# chpc
FROM --platform=linux/amd64 tonistiigi/xx:1.3.0 AS xx

###BUILD STAGE
# FROM --platform=$BUILDPLATFORM rust:${RUST_VERSION}-alpine AS build
# chpc
FROM --platform=linux/amd64 ubuntu:18.04 AS build

# Install additional build dependencies.
# include rust, cargo, and rustup and rustc 
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    clang \
    lld \
    git \
    file \
    rustc \
    cargo 

ARG APP_NAME
WORKDIR /app

# Copy cross compilation utilities from the xx stage.
COPY --from=xx / /
# Copy the hpo_data and hpoAssociations folders from src to the container
COPY /src/hpo_data/ /app/hpo_data/
COPY /src/hpoAssociations/ /app/hpoAssociations/
COPY bin_hpo_file /app/bin_hpo_file

COPY src /app/src
COPY Cargo.toml /app/
COPY Cargo.lock /app/

# This is the architecture you’re building for, which is passed in by the builder.
ARG TARGETPLATFORM

# Build the application.
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/,id=rust-cache-${APP_NAME}-${TARGETPLATFORM} \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release --target-dir ./target \
    && cp ./target/release/$APP_NAME /bin/server

## FINAL STAGE
# FROM alpine:3.18 AS final
# chpc
FROM --platform=linux/amd64 ubuntu:18.04 AS final

ARG UID=10001
RUN useradd -r -u ${UID} -g users appuser

# Copy the executable from the "build" stage.
COPY --from=build /bin/server /bin/
RUN chmod a+x /bin/server

# Copy the required directories from the "build" stage to the final stage.
COPY --from=build /app/hpo_data /hpo_data
COPY --from=build /app/hpoAssociations /hpoAssociations
COPY --from=build /app/bin_hpo_file /bin_hpo_file

# Change the owner of the /app directory to appuser
RUN chown -R appuser:users /hpo_data
RUN chown -R appuser:users /hpoAssociations
RUN chown -R appuser:users /bin_hpo_file

USER appuser

# Expose the port that the application listens on.
EXPOSE 8911

# What the container should run when it is started.
CMD ["/bin/server"]