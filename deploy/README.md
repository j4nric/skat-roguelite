# Deployment auf den vServer

Jeder Push auf `main` baut das Spiel und spielt es auf den vServer. Auf dem Server laufen zwei kleine Container: Caddy liefert die statischen Dateien aus und leitet `/api/*` an den Konto-Dienst weiter (ADR 0021). Gebaut wird alles in GitHub Actions, auf dem Server nichts.

Adresse: **https://stammtisch-62-83-23-90.sslip.io**

## Wie das Spiel erreichbar ist

Auf dem Server laufen bereits weitere Anwendungen. Ports 80 und 443 gehören dem Caddy der Haushalt-App (Repo `j4nric/haushalt-app`, Ordner `infra/production`). Dieser Caddy holt das HTTPS-Zertifikat für unsere Domain und leitet weiter:

```
Browser ──HTTPS──▶ haushalt-prod-caddy-1 ──Netzwerk stammtisch-proxy──▶ stammtisch-web (unser Caddy)
                                                                          │ /api/*
                                                                          ▼ Netzwerk backend (intern)
                                                                        stammtisch-api (Konto-Dienst, SQLite in ./data)
```

Der Haushalt-Caddy reicht alles an unseren Caddy weiter; für `/api` braucht es dort keine Änderung. Der Konto-Dienst veröffentlicht keine Ports und hängt nur im internen Netzwerk `backend`, das allein unser Caddy erreicht.

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
3. Web-App bauen (`web/`, `npm run build`: Kern als WebAssembly plus Vite-Build) und `site/` aus `web/dist/` zusammenstellen. Commit und Datum stehen als Build-Info in der Fußzeile des Startbildschirms. Der Build enthält `sw.js`, den Service Worker für das Spielen ohne Netz (DESIGN 14.5); Caddy liefert ihn mit `Cache-Control: no-cache` aus. Dass `rsync --delete` die Dateien des alten Builds löscht, schadet nicht: Offene Tabs der alten Version holen sie aus ihrem Cache, und die neue Version meldet sich in der App mit „Neue Version verfügbar – neu laden“.
4. Parallel dazu der Konto-Dienst (`server/`) als statisches musl-Binary (`x86_64-unknown-linux-musl`, SQLite eingebaut). Er läuft in einem schlichten `alpine`-Image, das Binary wird hineingemountet; auf dem Server wird nichts gebaut und kein eigenes Image gebraucht.
5. `site/`, `server/stammtisch-server`, `docker-compose.yml`, `Caddyfile` und `up.sh` per `rsync` über SSH nach `/opt/skat`. `data/` wird nie synchronisiert oder gelöscht.
6. `up.sh` auf dem Server: Netzwerk `stammtisch-proxy` anlegen, falls es fehlt; `data/` anlegen; `.env` mit der uid/gid des Deploy-Nutzers schreiben (der Dienst läuft als dieser Nutzer, die Datenbank gehört ihm); erst `web` starten (bei geänderter Caddyfile neu starten), dann `api` neu erzeugen und auf `/api/health` warten.

Startet der Konto-Dienst nicht oder scheitert sein Bau, gibt es nur eine Warnung: Das Spiel wird trotzdem ausgeliefert, `/api` antwortet 502 und die App zeigt beim Konto „offline“.

## Konto-Dienst und Datenbank (ADR 0021)

| Was | Wo |
|---|---|
| Binary | `/opt/skat/server/stammtisch-server` |
| Datenbank (SQLite, WAL) | `/opt/skat/data/stammtisch.db` (dazu `-wal` und `-shm`) |
| Einstellungen | `environment` in `docker-compose.yml`: `STAMMTISCH_ORIGIN` (Herkunft der Seite, daraus die Passkey-RP-Id), `STAMMTISCH_DB`, `STAMMTISCH_LISTEN` |

Einmalig ist nichts zu tun: `up.sh` legt `data/` selbst an. Secrets braucht der Dienst keine (Sitzungen sind Zufallstokens in der Datenbank).

**Sicherung.** Die Datenbank ist der einzige Zustand auf dem Server. Eine konsistente Kopie im laufenden Betrieb:

```
sqlite3 /opt/skat/data/stammtisch.db ".backup '/opt/skat/backup-$(date +%F).db'"
```

Ohne `sqlite3` auf dem Server: Dienst kurz anhalten und die Datei kopieren (die Seite läuft weiter, nur `/api` ist so lange weg):

```
cd /opt/skat && docker compose stop api && cp data/stammtisch.db "backup-$(date +%F).db" && docker compose start api
```

Zurückspielen: Dienst anhalten, Datei nach `data/stammtisch.db` kopieren (alte `-wal`/`-shm` löschen), Dienst starten.

**Domain wechseln.** Passkeys gelten nur für die Domain, auf der sie angelegt wurden. Nach einem Umzug `STAMMTISCH_ORIGIN` anpassen; Spieler melden sich einmal mit dem Wiederherstellungscode an und legen einen neuen Passkey an (ADR 0021).

**Lokal entwickeln.** `cargo run -p stammtisch_server` (lauscht auf `127.0.0.1:8787`, Datenbank `stammtisch.db` im aktuellen Ordner, erlaubt `http://localhost:5173` und `:4173`), dazu `npm run dev` oder `npm run preview` in `web/`: Vite leitet `/api` dorthin.

## Fehlersuche

```
cd /opt/skat && docker compose ps
cd /opt/skat && docker compose logs --tail 50
docker network inspect stammtisch-proxy --format '{{range .Containers}}{{.Name}} {{end}}'
```

Der letzte Befehl muss `stammtisch-web-1` und `haushalt-prod-caddy-1` zeigen. Fehlt der Caddy, ist der Haushalt-PR noch nicht ausgerollt.

Konto-Dienst:

```
cd /opt/skat && docker compose logs --tail 50 api
cd /opt/skat && docker compose exec -T api wget -qO- http://127.0.0.1:8080/api/health    # "ok"
curl -s https://stammtisch-62-83-23-90.sslip.io/api/health                                 # "ok" von außen
sh /opt/skat/up.sh                                                                          # alles neu starten
```
