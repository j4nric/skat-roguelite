#!/bin/sh
# Start or update the containers. The deploy job (.github/workflows/deploy.yml)
# syncs this file to $DEPLOY_PATH and runs it there after the files are in
# place. Safe to run by hand: `sh /opt/skat/up.sh`.
#
# The static site comes first and must work. The account server (ADR 0021)
# comes after it; if it does not start, this only warns: the game is served
# anyway and only /api answers 502.
set -eu
cd "$(dirname "$0")"

docker network inspect stammtisch-proxy >/dev/null 2>&1 || docker network create stammtisch-proxy
# Shared network of the front Caddy and all apps behind it (ADR 0024).
docker network inspect edge >/dev/null 2>&1 || docker network create edge
# data/ holds the account server's SQLite file; it is never synced or deleted.
mkdir -p data server
# The account server runs as the deploy user, so the database belongs to it.
printf 'STAMMTISCH_UID=%s\nSTAMMTISCH_GID=%s\n' "$(id -u)" "$(id -g)" > .env

# Newer patch releases of the pinned images (security fixes). Without a
# registry the old images keep running.
docker compose pull -q || echo "::warning::docker compose pull failed; running the images already on the server."

# Apple app site association for the iOS app's passkeys (ADR 0025). The
# deploy job hands over the repository variable APPLE_TEAM_ID (empty when
# unset); it is kept in apple-team-id, so a run by hand without it keeps
# the last one. rsync --delete-after removed the file with the old site;
# it is written anew on every run.
if [ -n "${APPLE_TEAM_ID+x}" ]; then printf '%s\n' "$APPLE_TEAM_ID" > apple-team-id; fi
APPLE_TEAM_ID=$(cat apple-team-id 2>/dev/null || true) sh ./aasa.sh site

docker compose up -d --remove-orphans web

# The Caddyfile is a single-file bind mount: rsync replaces the file, the
# running container keeps the old one. Restart Caddy when it changed.
sum=$(sha256sum Caddyfile | cut -d' ' -f1)
if [ "$sum" != "$(cat .caddyfile.sha256 2>/dev/null || true)" ]; then
  docker compose restart web
  echo "$sum" > .caddyfile.sha256
fi

# Our entry in the front Caddy (ADR 0024, deploy/site.caddy). Check it on
# its own, keep the previous one as .prev (not imported, no .caddy ending),
# put the new one into the shared volume, then let the front Caddy check
# the whole configuration and reload without a restart. If that check
# fails, the previous entry goes back and the deploy fails; the front Caddy
# keeps running with the configuration it had.
SITE_IMAGE=caddy:2.11-alpine
SITE_NAME=stammtisch.caddy
docker volume inspect caddy-sites >/dev/null 2>&1 || docker volume create caddy-sites >/dev/null
docker run --rm -v caddy-sites:/sites -v "$PWD/site.caddy:/new.caddy:ro" "$SITE_IMAGE" sh -euc "
  caddy validate --config /new.caddy --adapter caddyfile >/dev/null 2>&1 || { caddy validate --config /new.caddy --adapter caddyfile; exit 1; }
  if [ -f /sites/$SITE_NAME ]; then cp /sites/$SITE_NAME /sites/$SITE_NAME.prev; else rm -f /sites/$SITE_NAME.prev; fi
  cp /new.caddy /sites/$SITE_NAME
"
front=$(docker ps -q --filter label=edge.caddy=true | head -n 1)
if [ -z "$front" ]; then
  echo "::warning::No front Caddy with label edge.caddy=true yet; site entry stored in caddy-sites and picked up when it starts."
elif docker exec "$front" caddy validate --config /etc/caddy/Caddyfile --adapter caddyfile >/dev/null 2>&1; then
  docker exec "$front" caddy reload --config /etc/caddy/Caddyfile --adapter caddyfile
  echo "front Caddy: site entry loaded"
else
  docker exec "$front" caddy validate --config /etc/caddy/Caddyfile --adapter caddyfile || true
  docker run --rm -v caddy-sites:/sites "$SITE_IMAGE" sh -euc "
    if [ -f /sites/$SITE_NAME.prev ]; then mv /sites/$SITE_NAME.prev /sites/$SITE_NAME; else rm -f /sites/$SITE_NAME; fi
  "
  echo "::error::The front Caddy refused deploy/site.caddy (see above); the previous entry is back in place."
  exit 1
fi

# The binary is a bind mount as well: recreate the container only when it
# changed (or when compose sees a new image or config, which `up -d` does
# by itself). An unchanged deploy leaves the account server running.
ok=
started=
if [ -x server/stammtisch-server ]; then
  bin=$(sha256sum server/stammtisch-server | cut -d' ' -f1)
  if [ "$bin" != "$(cat .server.sha256 2>/dev/null || true)" ]; then
    docker compose up -d --force-recreate api && started=1 && echo "$bin" > .server.sha256
  else
    docker compose up -d api && started=1
  fi
fi
if [ -n "$started" ]; then
  for _ in 1 2 3 4 5 6 7 8 9 10; do
    sleep 1
    if docker compose exec -T api wget -qO- http://127.0.0.1:8080/api/health >/dev/null 2>&1; then
      ok=1
      break
    fi
  done
fi
if [ -n "$ok" ]; then
  echo "account server: ok"
else
  echo "::warning::Account server did not come up; the static site is served anyway. See deploy/README.md."
  docker compose logs --tail 30 api || true
fi

docker compose ps
