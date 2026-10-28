# Built from the repository root: docker build -f crates/anoma-telemetry-server/Dockerfile .
FROM rust:1-slim AS build
WORKDIR /src
# The root manifest only for the shared crate's workspace settings (edition, lints).
COPY Cargo.toml Cargo.toml
COPY crates/anoma-telemetry crates/anoma-telemetry
COPY crates/anoma-telemetry-server crates/anoma-telemetry-server
RUN cargo build --release --locked --manifest-path crates/anoma-telemetry-server/Cargo.toml

FROM debian:stable-slim
COPY --from=build /src/crates/anoma-telemetry-server/target/release/anoma-telemetry-server /usr/local/bin/
ENV ANOMA_TELEMETRY_DB=/data/telemetry.sqlite3 ANOMA_TELEMETRY_TRUST_PROXY=1
CMD ["anoma-telemetry-server"]
