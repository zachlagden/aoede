# Audio Pipeline Investigation (February 2026)

## Summary

The "no audio" issue is caused by **librespot 0.4.2 being fundamentally incompatible with current Spotify infrastructure**. The audio pipeline code itself is correct — the problem is upstream in the Spotify protocol layer.

## Build Issue (Fixed ✅)

The `audiopus_sys` crate uses an old `CMakeLists.txt` that's incompatible with newer CMake. Fixed by adding `ENV CMAKE_POLICY_VERSION_MINIMUM=3.5` to the Dockerfile.

## Audio Issue Analysis

### What works
- Bot starts and connects to Discord ✅
- Device appears in Spotify Connect ✅
- Bot joins voice channel when user is in one ✅

### What doesn't work
- No audio plays when a song is selected ❌
- Tracks either fail to load silently or skip immediately

### Root Cause: librespot 0.4.2 is obsolete

Aoede uses **librespot 0.4.2** (released ~2022). The current version is **0.8.0** (November 2025). Between these versions, Spotify made several breaking server-side changes:

#### 1. Password Authentication Deprecated (August 2024)
- Spotify deprecated username/password login in August 2024
- `Credentials::with_password(username, password)` no longer works reliably
- librespot 0.5.0+ added OAuth support as replacement
- Cached credentials may still work temporarily, but the initial login flow is broken
- References:
  - https://github.com/librespot-org/librespot/issues/1308
  - https://community.home-assistant.io/t/spotify-addon-doesnt-work-with-credentials/843459

#### 2. Metadata Endpoint Changes (November 2025)
- Spotify changed how track metadata is served
- Old Mercury-based endpoints return 403 errors
- librespot 0.8.0 fixed this by switching to `get_extended_metadata` (PR #1622)
- On librespot 0.4.2, tracks cannot be loaded because metadata resolution fails
- Reference: https://github.com/librespot-org/librespot/issues/1623

#### 3. Access Point Resolution (apresolve)
- `apresolve.spotify.com` returns access points incompatible with old protocol versions
- Blocking this endpoint forces fallback to `ap.spotify.com` (workaround added)
- Reference: https://github.com/librespot-org/librespot/issues/972

#### 4. Protocol Migration: Mercury → HTTP
- librespot 0.5.0 was a major rewrite moving from Mercury to HTTP-based APIs
- File downloads switched from the old channel system to HTTPS CDN
- The old Mercury-based SPIRC message bus was replaced with WebSockets (dealer)
- librespot 0.4.2 uses the old Mercury protocol which Spotify is sunsetting

### Audio Pipeline Code Review

The audio pipeline in `src/lib/player.rs` is actually **correctly implemented** for librespot 0.4.2:

```
Spotify → librespot (AudioPacket: Vec<f64>) → EmittedSink (resample 44.1kHz→48kHz)
  → sync_channel → songbird (FloatPcm) → Discord voice
```

- `AudioPacket::Samples(Vec<f64>)` values are normalized [-1.0, 1.0]
- Cast to `f32` loses some precision but is functionally correct
- `FftFixedInOut` resampler correctly converts from librespot's 44.1kHz to songbird's 48kHz
- `sync_channel(1120)` provides appropriate backpressure
- songbird `input::codec::Codec::FloatPcm` matches the f32 output format
- The blocking `recv()` on first sample prevents songbird from thinking the track ended

**The pipeline never receives data because librespot can't fetch tracks from Spotify.**

### What the apresolve workaround does

The entrypoint script blocks `apresolve.spotify.com` via `/etc/hosts`, forcing librespot to use fallback access points. This fixes one layer of the problem (connectivity) but **does not fix** the deprecated authentication or changed metadata endpoints.

**This workaround may partially restore functionality** for users who already have valid cached credentials, but it is not a reliable long-term fix.

## Recommended Fix: Upgrade to librespot 0.8.0

The only real fix is upgrading librespot. This is a **major refactor** because of extensive breaking changes:

### What needs to change

| Component | Current | Target | Breaking Changes |
|-----------|---------|--------|-----------------|
| librespot | 0.4.2 | 0.8.0 | Entire API rewritten |
| songbird | 0.3.1 | 0.4.x | Input API changed |
| serenity | 0.11.2 | 0.12.x | Gateway intents, cache API |
| rubato | 0.12.0 | latest | Minor API changes |

### Key librespot 0.5+ changes affecting aoede:
- `Session::connect()` split into `Session::new()` + `session.connect()` 
- `Credentials.username` is now `Option<String>`
- `Sink::write()` now takes ownership of the packet
- `AudioPacket` structure changed (Symphonia decoder replaces lewton)
- OAuth authentication required (password login removed)
- `SpotifyId` replaced by `SpotifyUri` in 0.8.0
- `DeviceType` moved from `connect` to `core`
- Metadata retrieval completely rewritten

### Estimated effort
- **High** — effectively a rewrite of `player.rs` and significant changes to `main.rs`
- ~2-4 days for an experienced Rust developer
- The audio pipeline (EmittedSink) would need moderate changes for the new Sink API
- Authentication flow needs complete replacement (OAuth instead of password)

## Alternative: Fork with newer librespot

Consider checking if any community forks of aoede have already been updated. The upstream `codetheweb/aoede` appears abandoned (last meaningful update was 2022).

## Dependencies (Current)

```toml
librespot = "0.4.2"    # ← 4 major versions behind (current: 0.8.0)
songbird = "0.3.1"     # ← 1 major version behind (current: 0.4.x)
serenity = "0.11.2"    # ← 1 major version behind (current: 0.12.x)
rubato = "0.12.0"      # ← likely fine
tokio = "1.20.1"       # ← works but outdated
```
