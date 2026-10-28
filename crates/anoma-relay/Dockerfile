# The Play with friends relay, for anyone to run (docs/play-with-friends.md, "Running your own relay").
# Build context: the repository root (the workspace's manifests and lockfile):
#   docker build -f crates/anoma-relay/Dockerfile -t anoma-relay .
FROM rust:1-slim-bookworm AS build
WORKDIR /src
COPY . .
RUN cargo build --release --locked -p anoma-relay \
    && cp target/release/anoma-relay /anoma-relay

FROM debian:bookworm-slim
COPY --from=build /anoma-relay /usr/local/bin/anoma-relay
ENV PORT=7000
EXPOSE 7000
USER nobody
CMD ["anoma-relay"]
