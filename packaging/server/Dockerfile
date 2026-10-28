# anoma-server in a container (docs/server.md, section 3). Build context: the repository root:
#   docker build -f packaging/server/Dockerfile -t anoma-server .
#
# The image holds none of Minecraft's files. Prepare them on the host from your own Minecraft 26.3
# (anoma-prepare --prepare, see packaging/PLAY.txt) and mount the folder it writes, read-only:
#   docker run -d --name anoma -p 25565:25565 --stop-timeout 180 \
#     -v anoma-data:/data \
#     -v /path/to/.local/anoma-client/26.3:/opt/anoma/.local/anoma-client/26.3:ro \
#     -e ANOMA_EULA=true -e ANOMA_WHITELIST=true -e ANOMA_PLAYERS=Alex,Steve -e ANOMA_OP=Alex \
#     anoma-server
#
# ANOMA_EULA=true accepts the Minecraft EULA (https://aka.ms/MinecraftEULA) for this server. The first
# start writes /data/server.toml from ANOMA_* (EULA, WORLD, SEED, PORT, WHITELIST, PLAYERS, OP); later
# starts use the file. Manage it with `docker exec anoma anoma-server status|cmd|backup|logs ...`.
FROM rust:1-slim-bookworm AS build
WORKDIR /src
COPY . .
# The server build of the game (no sound, no controllers: it needs neither ALSA nor udev) as anoma-engine.
RUN cargo build --release --locked -p anoma-client --no-default-features --bin anoma-client \
    && cargo build --release --locked -p anoma-server --bin anoma-server \
    && mkdir /out && cp target/release/anoma-client /out/anoma-engine && cp target/release/anoma-server /out/

FROM debian:bookworm-slim
RUN useradd --system --home-dir /data --shell /usr/sbin/nologin anoma \
    && mkdir -p /data /opt/anoma/.local/anoma-client \
    && chown anoma:anoma /data
COPY --from=build /out/ /opt/anoma/
ENV PATH=/opt/anoma:$PATH \
    ANOMA_NO_AUDIO=1
WORKDIR /data
VOLUME /data
EXPOSE 25565
USER anoma
# SIGTERM makes anoma-server stop the game cleanly (it saves first): give it --stop-timeout 180.
CMD ["sh", "-c", "test -f server.toml || anoma-server init --yes || exit 1; exec anoma-server run"]
