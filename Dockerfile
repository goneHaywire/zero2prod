# Builder stage

# base image
FROM lukemathwalker/cargo-chef:latest-rust-1.94.1 AS chef
# set workdir
WORKDIR /app
# install required pkgs
RUN apt update && apt install lld-19 clang-19 -y

FROM chef AS planner
# copy everything to folder
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
# make sqlx get database metadata offile
ENV SQLX_OFFLINE=true
# build binary in release profile
RUN cargo build --release --bin zero2prod

# Runtime stage
FROM debian:trixie-slim AS runtime
WORKDIR /app

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/zero2prod zero2prod

COPY configuration configuration
# set prod env
ENV APP_ENVIRONMENT=production

# run binary
ENTRYPOINT [ "./zero2prod" ]
