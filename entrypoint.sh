#!/bin/sh
# Workaround: block apresolve.spotify.com to force librespot to use fallback
# access points. Spotify changed its infrastructure and the endpoints returned
# by apresolve are incompatible with older librespot versions (0.4.x).
# This forces librespot to fall back to ap.spotify.com:443.
# See: https://github.com/librespot-org/librespot/issues/972
echo "0.0.0.0 apresolve.spotify.com" >> /etc/hosts 2>/dev/null || true

exec /usr/local/bin/aoede "$@"
