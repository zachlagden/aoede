<p align="center">
  <img width="250" height="250" src=".github/logo.png">
</p>

# Aoede

Aoede is a Discord music bot that streams from Spotify to Discord. The only playback interface is Spotify itself.

This fork reads cached Spotify credentials before trying username/password authentication. Build this fork locally to include that support. The upstream `codetheweb/aoede` image and upstream release binaries do not contain these changes.

Spotify Premium is required by librespot, the Spotify library Aoede uses. Facebook logins [are not supported](https://github.com/librespot-org/librespot/discussions/635). Authentication and playback depend on Spotify and Discord compatibility with the existing librespot and Songbird versions.

![Demo](.github/demo.gif)

## Uses

- Small servers with friends
- Discord Stages for broadcasting music

## Setup

Aoede requires a Discord bot token, not a user token. It follows the user specified by `DISCORD_USER_ID` and enables Spotify Connect when it detects that user in a voice channel.

Clone this fork and run the following commands from the checkout directory:

```bash
git clone https://github.com/zachlagden/aoede.git
cd aoede
mkdir -p aoede-cache
```

Follow [cached credentials setup](#cached-credentials-setup) to place `credentials.json` in `aoede-cache`. Keep this file private. Do not commit it or put it in an image.

Create a local `.env` file with these values, replacing the token and user ID:

```dotenv
DISCORD_TOKEN=replace_with_your_discord_bot_token
DISCORD_USER_ID=123456789
SPOTIFY_BOT_AUTOPLAY=false
SPOTIFY_DEVICE_NAME=Aoede
```

`DISCORD_USER_ID` must be the numeric ID of the user Aoede should follow. Do not add `SPOTIFY_USERNAME` or `SPOTIFY_PASSWORD` when using cached credentials. `.env`, `config.toml`, and `aoede-cache` are ignored by Git and excluded from the Docker build context.

### Docker Compose

Save this as `compose.yaml` in the fork checkout, alongside `Dockerfile`. The build context must be that checkout, not a separate directory containing only the Compose file.

```yaml
services:
  aoede:
    build:
      context: .
    image: aoede:local
    restart: unless-stopped
    volumes:
      - ./aoede-cache:/data
    environment:
      DISCORD_TOKEN: ${DISCORD_TOKEN:?Set DISCORD_TOKEN in .env}
      DISCORD_USER_ID: ${DISCORD_USER_ID:?Set a numeric DISCORD_USER_ID in .env}
      CACHE_DIR: /data
      SPOTIFY_BOT_AUTOPLAY: ${SPOTIFY_BOT_AUTOPLAY:-false}
      SPOTIFY_DEVICE_NAME: ${SPOTIFY_DEVICE_NAME:-Aoede}
```

Build and start it:

```bash
docker compose up --build -d
docker compose logs --tail=100 aoede
```

The local image is named `aoede:local`. The bind mount preserves credentials and cached audio across container replacements. The container must be able to read and write `aoede-cache`.

### Docker

Use the same `.env` and credential-cache directory:

```bash
docker build --tag aoede:local .
docker run -d --name aoede --restart unless-stopped \
  --env-file .env \
  --mount type=bind,source="$(pwd)/aoede-cache",target=/data \
  aoede:local
docker logs --tail=100 aoede
```

The Dockerfile sets `CACHE_DIR=/data`. Mount `aoede-cache` there so the bot can load `credentials.json` and keep its cache between runs. No fork image download is required.

### Building from source

Use the same [fork checkout](https://github.com/zachlagden/aoede) rather than an upstream release binary. These instructions do not require a prebuilt fork binary.

Requirements:

- Rust and Cargo
- A C/C++ compiler and make
- automake, autoconf, and libtool
- CMake
- pkg-config

Build the checked-in dependency versions:

```bash
CMAKE_POLICY_VERSION_MINIMUM=3.5 cargo build --locked --release
cp config.sample.toml config.toml
```

Set your Discord bot token and numeric user ID in the local `config.toml`. Its default `cache_dir` is `aoede-cache`. Then run:

```bash
./target/release/aoede
```

The CMake setting allows the bundled Opus dependency to build with CMake 4. The Docker build sets it automatically. A successful build does not verify live Spotify authentication or Discord playback.

## Cached credentials setup

Spotify has deprecated username/password authentication. Use a credential cache rather than supplying those fields.

Download and run `librespot-auth` on x86_64 Linux:

```bash
wget https://github.com/dspearson/librespot-auth/releases/download/v0.1.1/librespot-auth-x86_64-linux-musl-static.tar.xz
tar -xf librespot-auth-x86_64-linux-musl-static.tar.xz
./librespot-auth-x86_64-linux-musl-static/librespot-auth --name "Aoede Bot"
```

For other platforms, see the tool's [releases](https://github.com/dspearson/librespot-auth/releases).

Open Spotify on your phone or computer and select "Aoede Bot" in the device picker. Once the tool writes `credentials.json`, stop it and copy that file into the persistent cache directory:

```bash
mkdir -p aoede-cache
cp credentials.json aoede-cache/credentials.json
```

If the tool writes the file elsewhere, use that path as the copy source. For Docker, Aoede reads the mounted file as `/data/credentials.json`. For a source build, it reads `aoede-cache/credentials.json` unless you change `CACHE_DIR` or `cache_dir`.

### Configuration

For source builds, copy and edit `config.sample.toml`. For Docker, use the environment settings above. Environment variables override `config.toml` values.

| Variable | Required | Description |
|----------|----------|-------------|
| `DISCORD_TOKEN` | Yes | Discord bot token |
| `DISCORD_USER_ID` | Yes | Numeric Discord user ID to follow |
| `CACHE_DIR` | No | Cache directory. Defaults to `aoede-cache` for source builds and `/data` in Docker |
| `SPOTIFY_USERNAME` | No | Legacy Spotify username, used only if cached credentials are unavailable |
| `SPOTIFY_PASSWORD` | No | Legacy Spotify password, used only if cached credentials are unavailable |
| `SPOTIFY_BOT_AUTOPLAY` | No | Autoplay similar songs when music ends. Defaults to `false` |
| `SPOTIFY_DEVICE_NAME` | No | Spotify device name. Defaults to `Aoede` |

Username/password remains an optional fallback in the source, but it should not be used for new setups.

### Troubleshooting

- `Missing field: 'SPOTIFY_USERNAME'` indicates an older build that requires username/password. Rebuild from this fork and use `aoede:local`, not the upstream image.
- `No cached credentials found` means the bot did not load a credential file. Check `aoede-cache/credentials.json`, the `/data` mount, and file permissions. Without cached credentials, Aoede tries the legacy username/password fallback.
- If the credential-generation device does not appear in Spotify, check that the tool and Spotify are on the same network.
- If cached credentials are rejected, regenerate them. If authentication or playback still fails, report the image's source revision and redacted logs. Never share `.env`, `config.toml`, `credentials.json`, tokens, or passwords.
