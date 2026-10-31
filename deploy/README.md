# Deployment auf den vServer

Jeder Push auf `main` baut das Spiel und spielt es auf den vServer. Der Server selbst führt nur `docker compose` aus, alles andere passiert in GitHub Actions.

## Was auf dem Server landet

```
$DEPLOY_PATH/
├── docker-compose.yml   # aus deploy/
├── Caddyfile            # aus deploy/
├── .env                 # SITE_ADDRESS, einmalig von Hand angelegt
└── site/                # statische Dateien: Platzhalter, ab M3 der Godot-Web-Export
```

Caddy liefert `site/` aus. Mit einer Domain in `.env` holt Caddy automatisch ein TLS-Zertifikat.

## Einmalige Einrichtung auf dem Server

1. Docker und Docker Compose installieren (`docker compose version` muss funktionieren).
2. Einen Nutzer für den Deploy anlegen, der Docker ausführen darf, z. B. `deploy` in der Gruppe `docker`.
3. Zielordner anlegen: `mkdir -p /opt/skat && chown deploy /opt/skat`.
4. SSH-Schlüsselpaar für den Deploy erzeugen (`ssh-keygen -t ed25519 -f skat-deploy`). Den öffentlichen Schlüssel in `~deploy/.ssh/authorized_keys` eintragen.
5. Optional `/opt/skat/.env` anlegen mit `SITE_ADDRESS=deine.domain.de`. Ohne diese Datei läuft der Server auf Port 80 ohne TLS.
6. Ports 80 und 443 in der Firewall öffnen.

## GitHub Secrets

Im Repository unter Settings → Secrets and variables → Actions:

| Secret | Inhalt |
|---|---|
| `DEPLOY_HOST` | Hostname oder IP des vServers |
| `DEPLOY_USER` | SSH-Nutzer, z. B. `deploy` |
| `DEPLOY_SSH_KEY` | Privater Schlüssel aus Schritt 4, vollständig inklusive Kopf- und Fußzeile |
| `DEPLOY_PATH` | Zielordner, z. B. `/opt/skat` |

Solange die Secrets fehlen, überspringt der Deploy-Job den Server-Schritt und meldet das im Workflow-Log. CI bleibt grün.

## Ablauf des Workflows (`.github/workflows/deploy.yml`)

1. Tests und Lint wie in `ci.yml`.
2. Simulator-Baseline erzeugen.
3. `site/` zusammenstellen: aktuell der Platzhalter aus `web/` mit Build-Info, ab M3 der Godot-Web-Export.
4. `site/`, `docker-compose.yml` und `Caddyfile` per `rsync` über SSH auf den Server.
5. `docker compose up -d` auf dem Server. Caddy nimmt neue Dateien ohne Neustart auf.

## Manuell auslösen

Unter Actions → Deploy → Run workflow. Das baut und deployt den aktuellen Stand von `main`.
