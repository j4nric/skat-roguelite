# Deployment auf den vServer

Jeder Push auf `main` baut das Spiel und spielt es auf den vServer. Auf dem Server läuft nur ein kleiner Caddy-Container, der die statischen Dateien ausliefert. Alles andere passiert in GitHub Actions.

Adresse: **https://stammtisch-62-83-23-90.sslip.io**

## Wie das Spiel erreichbar ist

Auf dem Server laufen bereits weitere Anwendungen. Ports 80 und 443 gehören dem Caddy der Haushalt-App (Repo `j4nric/haushalt-app`, Ordner `infra/production`). Dieser Caddy holt das HTTPS-Zertifikat für unsere Domain und leitet weiter:

```
Browser ──HTTPS──▶ haushalt-prod-caddy-1 ──Netzwerk stammtisch-proxy──▶ stammtisch-web (unser Container)
```

Dafür braucht es drei Dinge, die zusammenpassen müssen:

| Was | Wo |
|---|---|
| Netzwerk `stammtisch-proxy` | auf dem Server, einmalig angelegt |
| Block für `stammtisch-62-83-23-90.sslip.io` und das Netzwerk im Caddy | `haushalt-app/infra/production/Caddyfile` und `docker-compose.yml` |
| Unser Container mit Alias `stammtisch-web` im Netzwerk | `deploy/docker-compose.yml` in diesem Repo |

Die Domain kommt von sslip.io: Jede Subdomain mit eingebetteter IP zeigt automatisch auf diese IP. Sie kostet nichts und braucht keine Registrierung. Eine eigene Domain kann später den Block im Haushalt-Caddy ersetzen.

## Einmalige Einrichtung, in dieser Reihenfolge

**1. Netzwerk anlegen.** Muss vor Schritt 2 passieren, sonst scheitert der nächste Haushalt-Deploy.

```
docker network create stammtisch-proxy
```

**2. Haushalt-Caddy erweitern.** Den PR im Repo `j4nric/haushalt-app` mergen, der das Netzwerk und den Block für unsere Domain einträgt. Der Haushalt-Deploy startet den Caddy dann mit dem neuen Netzwerk neu.

**3. Deploy-Nutzer und Schlüssel** auf dem Server:

```
sudo adduser --disabled-password --gecos "" deploy
sudo usermod -aG docker deploy
sudo mkdir -p /opt/skat && sudo chown deploy:deploy /opt/skat
sudo -u deploy mkdir -p -m 700 /home/deploy/.ssh
sudo -u deploy ssh-keygen -t ed25519 -N "" -f /home/deploy/.ssh/skat-deploy
sudo -u deploy sh -c 'cat /home/deploy/.ssh/skat-deploy.pub >> /home/deploy/.ssh/authorized_keys && chmod 600 /home/deploy/.ssh/authorized_keys'
sudo cat /home/deploy/.ssh/skat-deploy     # in das Secret DEPLOY_SSH_KEY kopieren
sudo rm /home/deploy/.ssh/skat-deploy      # danach vom Server löschen
```

**4. GitHub Secrets** unter https://github.com/j4nric/skat-roguelite/settings/secrets/actions:

| Secret | Wert |
|---|---|
| `DEPLOY_HOST` | `62.83.23.90` |
| `DEPLOY_USER` | `deploy` |
| `DEPLOY_SSH_KEY` | privater Schlüssel aus Schritt 3, komplett mit BEGIN- und END-Zeile |
| `DEPLOY_PATH` | `/opt/skat` |

Solange die Secrets fehlen, überspringt der Deploy-Job den Server-Schritt mit einer Warnung.

## Ablauf des Workflows (`.github/workflows/deploy.yml`)

1. Tests und Lint wie in `ci.yml`.
2. Simulator-Baseline erzeugen.
3. `site/` zusammenstellen: aktuell der Platzhalter aus `web/` mit Build-Info, ab M3 der Godot-Web-Export.
4. `site/`, `docker-compose.yml` und `Caddyfile` per `rsync` über SSH nach `/opt/skat`.
5. Netzwerk `stammtisch-proxy` anlegen, falls es fehlt, dann `docker compose up -d`.

## Fehlersuche

```
cd /opt/skat && docker compose ps
cd /opt/skat && docker compose logs --tail 50
docker network inspect stammtisch-proxy --format '{{range .Containers}}{{.Name}} {{end}}'
```

Der letzte Befehl muss `stammtisch-web-1` und `haushalt-prod-caddy-1` zeigen. Fehlt der Caddy, ist der Haushalt-PR noch nicht ausgerollt.
