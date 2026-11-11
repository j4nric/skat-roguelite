#!/bin/sh
# Write the Apple app site association file into the static site (ADR 0025):
#
#   APPLE_TEAM_ID=ABCDE12345 sh aasa.sh site
#
# The iOS app (Capacitor, bundle id de.onemoreskat.app) uses the passkeys of
# one-more-skat.de through Associated Domains; iOS fetches
# https://one-more-skat.de/.well-known/apple-app-site-association (JSON, no
# redirect) and checks that it names the app. Without a valid team id (ten
# letters or digits) the file is removed, so the address answers 404 rather
# than with a broken file. deploy/Caddyfile sets its Content-Type.
set -eu
site=${1:?usage: aasa.sh <site dir>}
team=${APPLE_TEAM_ID:-}
file="$site/.well-known/apple-app-site-association"
if printf '%s' "$team" | grep -Eq '^[A-Z0-9]{10}$'; then
  mkdir -p "$site/.well-known"
  printf '{"webcredentials":{"apps":["%s.de.onemoreskat.app"]}}\n' "$team" > "$file.tmp"
  mv "$file.tmp" "$file"
  echo "apple-app-site-association: $team.de.onemoreskat.app"
else
  rm -f "$file"
  if [ -n "$team" ]; then
    echo "::warning::APPLE_TEAM_ID is not ten letters or digits; no apple-app-site-association."
  else
    echo "apple-app-site-association: no APPLE_TEAM_ID yet (404)"
  fi
fi
