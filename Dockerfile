# Built in CI only, where imports/shared-libs is a checkout, not a link.
FROM rust:1-bookworm AS build
WORKDIR /src
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/application-a /usr/local/bin/application-a
USER 10001
EXPOSE 8080
ENTRYPOINT ["application-a"]
