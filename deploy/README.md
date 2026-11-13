# Deployment auf den vServer

Jeder Push auf `main` baut das Spiel und spielt es auf den vServer. Auf dem Server laufen zwei kleine Container: Caddy liefert die statischen Dateien aus und leitet `/api/*` an den Konto-Dienst weiter (ADR 0021). Gebaut wird alles in GitHub Actions, auf dem Server nichts.

Adresse: **https://one-more-skat.de** (alte Adresse https://stammtisch-62-83-23-90.sslip.io bleibt ohne Umleitung, mit Umzugshinweis, ADR 0025)

**Adressen der Seite** (`Caddyfile`, DESIGN 16.9): `/` ist die Startseite für Erstbesucher (`start.html`, englisch `/en`), `/spielen` das Spiel (`index.html`), daneben `/impressum`, `/datenschutz`, `/support`, `/presse` und ihre englischen Seiten unter `/en/`. Nur Umschreibungen ohne Umleitung, jede Seite mit `Cache-Control: no-cache`; ein unbekannter Pfad bleibt 404. Wer schon gespielt hat, wird von der Startseite selbst (`/landing.js`) ins Spiel geschickt. Die CI prüft die Adressen gegen einen echten Caddy (`ci.yml`, Schritt „Apple app site association and static pages“).

## Wie das Spiel erreichbar ist

Auf dem Server laufen bereits weitere Anwendungen. Ports 80 und 443 gehören dem Caddy der Haushalt-App (Repo `j4nric/haushalt-app`, Ordner `infra/production`). Dieser Caddy holt das HTTPS-Zertifikat für unsere Domain und leitet weiter:

```
Browser ──HTTPS──▶ haushalt-prod-caddy-1 ──Netzwerk stammtisch-proxy──▶ stammtisch-web (unser Caddy)
                                                                          │ /api/*
                                                                          ▼ Netzwerk backend (intern)
                                                                        stammtisch-api (Konto-Dienst, SQLite in ./data)
```

Der Haushalt-Caddy reicht alles an unseren Caddy weiter; für `/api` braucht es dort keine Änderung. Der Konto-Dienst veröffentlicht keine Ports und hängt nur im internen Netzwerk `backend`, das allein unser Caddy erreicht.

Seit ADR 0024 bringt das Spiel seinen Eintrag für den Haushalt-Caddy selbst mit:

| Was | Wo |
|---|---|
| Netzwerk `edge` (gemeinsam für alle Apps) und Volume `caddy-sites` | auf dem Server, legt `up.sh` bei Bedarf an |
| Unser Eintrag mit allen Domains | `deploy/site.caddy`; `up.sh` legt ihn als `stammtisch.caddy` ins Volume und lädt den Haushalt-Caddy neu |
| Import der Einträge (`import /etc/caddy/sites/*.caddy`), Label `edge.caddy=true`, Netzwerk `edge` | einmalig in `haushalt-app/infra/production/Caddyfile` und `docker-compose.yml` |
| Unser Container mit Alias `stammtisch-web` in `edge` (und bis zum Umzug in `stammtisch-proxy`) | `deploy/docker-compose.yml` in diesem Repo |

**Domain ändern:** nur `deploy/site.caddy` anpassen und mergen. Der Deploy prüft den Eintrag einzeln und dann zusammen mit allen anderen im Haushalt-Caddy; lehnt der ihn ab (etwa eine Domain, die schon eine andere App nutzt), kommt der vorige Eintrag zurück, der Deploy wird rot, und der Haushalt-Caddy läuft unverändert weiter. Jeder Name darf auf dem Server nur einmal vorkommen.

**Erste Umstellung (11.11.2026):** Erst dieses Repo ausrollen (Netzwerk, Volume, Eintrag), dann den Haushalt-PR, der den alten sslip-Block entfernt und den Import einschaltet. Findet `up.sh` noch keinen Caddy mit Label, legt es den Eintrag nur ab und warnt.

Domains: **one-more-skat.de** (eigene Domain, `www` leitet um) und weiterhin **stammtisch-62-83-23-90.sslip.io**, bis die Spielstände im Browser umgezogen sind.

**Umzug (ADR 0025).** Die alte Adresse wird nicht umgeleitet: Ihr localStorage und ihre Passkeys gehören zu ihr. Der Konto-Dienst nimmt beide Adressen als eigene WebAuthn-Relying-Party (`STAMMTISCH_ORIGIN` in `docker-compose.yml`, die neue zuerst). Auf der alten Adresse zeigt das Spiel „Wir sind umgezogen“ und nimmt Konto (einmaliger Umzugs-Code, 10 Minuten) oder Spielstand (im Fragment des Links) mit.

**Apple-Datei für die iOS-App.** `/.well-known/apple-app-site-association` schreibt `deploy/aasa.sh` bei jedem Deploy aus der Repository-Variable `APPLE_TEAM_ID` (GitHub: Settings → Secrets and variables → Actions → Variables; zehn Großbuchstaben oder Ziffern). Ohne sie antwortet die Adresse 404. Auf dem Server merkt sich `up.sh` den Wert in `apple-team-id`, damit ein Aufruf von Hand ihn behält.

**iOS-App (ADR 0026).** Die App lädt das Spiel aus ihrem Paket (Ursprung `capacitor://localhost`) und spricht `https://one-more-skat.de/api` mit einem Bearer-Token statt des Cookies. Dafür steht `STAMMTISCH_APP_ORIGIN=capacitor://localhost` in `docker-compose.yml` (CORS nur für diesen Ursprung). Bau, Signatur und TestFlight: [`docs/ios.md`](../docs/ios.md).

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
| `DEPLOY_KNOWN_HOSTS` | freiwillig, aber empfohlen: die Zeile mit dem Host-Schlüssel des Servers (siehe unten) |

Solange die ersten vier Secrets fehlen, überspringt der Deploy-Job den Server-Schritt mit einer Warnung.

**Host-Schlüssel (`DEPLOY_KNOWN_HOSTS`).** Ohne dieses Secret vertraut der Workflow dem Schlüssel, den `ssh-keyscan` beim Deploy gerade sieht (mit einer Warnung im Lauf); ein Angreifer zwischen GitHub und Server fiele so nicht auf. Mit dem Secret prüft SSH streng (`StrictHostKeyChecking yes`). Die Zeile holen und gegen den Fingerabdruck vergleichen, den der Server selbst nennt:

```
ssh-keyscan -t ed25519 62.83.23.90                    # auf dem eigenen Rechner: die Zeile "62.83.23.90 ssh-ed25519 AAAA…"
ssh-keyscan -t ed25519 62.83.23.90 | ssh-keygen -lf -  # ihr Fingerabdruck (SHA256:…)
ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub      # auf dem Server (in der Konsole des Anbieters): muss gleich sein
```

Stimmen die Fingerabdrücke überein, die ganze Zeile `62.83.23.90 ssh-ed25519 AAAA…` als Secret `DEPLOY_KNOWN_HOSTS` anlegen. Bekommt der Server einen neuen Host-Schlüssel (Neuinstallation), schlägt der Deploy fehl, bis das Secret erneuert ist.

## Ablauf des Workflows (`.github/workflows/deploy.yml`)

Gebaut und getestet wird nur einmal, im CI-Workflow (`ci.yml`): Bei einem Pull Request läuft er für sich, bei einem Push auf `main` ruft der Deploy ihn auf (`workflow_call`) und liefert genau die Artefakte aus, die dort getestet wurden.

1. **CI** (`ci.yml`), parallel: `fmt`, Clippy und Tests (`check`); danach die Simulator-Baseline (`sim-baseline`, Seed 42 mit 10 000 Spielen, dazu 200 Runs mit Zeitmessung). Die Web-App (`web`): Kern als WebAssembly plus Vite-Build, Commit und Datum als Build-Info in der Fußzeile des Startbildschirms (Artefakt `web-dist`). Der Konto-Dienst (`server`) als statisches musl-Binary (`x86_64-unknown-linux-musl`, SQLite eingebaut, Profil `server-release` ohne Symbole), dazu `caddy validate` und `docker compose config` (Artefakt `server`).
2. **Deploy** (nur auf `main`, nie zwei gleichzeitig): lädt `web-dist`, `sim-baseline` und `server`. `site/` ist `web-dist` plus `sim-baseline.txt`. Der Build enthält `sw.js`, den Service Worker für das Spielen ohne Netz (DESIGN 14.5); Caddy liefert ihn mit `Cache-Control: no-cache` aus.
3. `site/` per `rsync --delete-after --delay-updates`: Neue Dateien landen erst unter vorläufigen Namen und werden am Ende zusammen umbenannt, alte erst danach gelöscht; eine Seite, die währenddessen lädt, sieht nie einen halben Build. Offene Tabs der alten Version holen gelöschte Dateien aus ihrem Cache, und die neue Version meldet sich in der App mit „Neue Version verfügbar – neu laden“. Dazu `server/stammtisch-server`, `docker-compose.yml`, `Caddyfile` und `up.sh` nach `/opt/skat`. `data/` wird nie synchronisiert oder gelöscht.
4. `up.sh` auf dem Server: Netzwerk `stammtisch-proxy` anlegen, falls es fehlt; `data/` anlegen; `.env` mit der uid/gid des Deploy-Nutzers schreiben (der Dienst läuft als dieser Nutzer, die Datenbank gehört ihm); `docker compose pull` holt neue Patch-Stände der Images (scheitert das, laufen die alten weiter); erst `web` starten (bei geänderter Caddyfile neu starten), dann `api` (neu erzeugt nur, wenn sich die SHA-256 des Binarys geändert hat) und auf `/api/health` warten.

Der Deploy-Job läuft auch, wenn CI rot ist, und hängt an den Artefakten: Fehlen `web-dist` oder `sim-baseline` (Build gescheitert oder Tests rot), scheitert er, ohne etwas auszuliefern. Fehlt nur `server` (Bau des Konto-Diensts gescheitert), gibt es eine Warnung: Das Spiel wird trotzdem ausgeliefert, das alte Binary bleibt, und fehlt auch das, antwortet `/api` 502 und die App zeigt beim Konto „offline“.

**Images.** `caddy:2.11-alpine` und `alpine:3.24` sind auf die Minor-Version festgelegt; Patch-Stände (Sicherheitskorrekturen) kommen mit dem `pull` bei jedem Deploy. Eine neue Minor-Version wird in `docker-compose.yml` und für `caddy validate` in `ci.yml` zusammen eingetragen.

**Container.** `api` läuft mit schreibgeschütztem Dateisystem (beschreibbar sind nur `./data` und ein `/tmp` im Speicher), ohne Capabilities (`cap_drop: ALL`), mit `no-new-privileges` und einem Healthcheck auf `/api/health` (`docker compose ps` zeigt `healthy`). `web` (Caddy) läuft unverändert als root im offiziellen Image. Die Logs beider Container rotieren (`local`, höchstens 3 Dateien zu 10 MB).

**Sicherheits-Header** (`Caddyfile`, für die ganze Seite): `Content-Security-Policy`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, `Permissions-Policy` (keine Kamera, kein Mikrofon, kein Standort); der `Server`-Header fällt weg. Die CSP erlaubt nur Eigenes, dazu was die App wirklich braucht: `'wasm-unsafe-eval'` für den Kern als WebAssembly, `'unsafe-inline'` bei `style-src` für die `style`-Attribute der Oberfläche, `data:` und `blob:` bei `img-src` für Porträts, Karten und die Run-Karte. Geprüft am 09.11.2026 in Chromium mit genau diesem Header: Startbildschirm, ein ganzes Spiel mit Ton und Bewegung, Service Worker, Konto mit Passkey, ohne einen einzigen Verstoß. Braucht eine neue Funktion mehr (etwa eine fremde Quelle), zeigt die Browser-Konsole „Refused to …“; dann die CSP hier anpassen.

## Konto-Dienst und Datenbank (ADR 0021)

| Was | Wo |
|---|---|
| Binary | `/opt/skat/server/stammtisch-server` |
| Datenbank (SQLite, WAL) | `/opt/skat/data/stammtisch.db` (dazu `-wal` und `-shm`) |
| Tägliche Sicherung | `/opt/skat/data/backup/stammtisch-JJJJMMTT.db` (die letzten sieben) |
| Fehlerberichte | Tabelle `reports` in derselben Datenbank, 90 Tage (ADR 0027, Abschnitt „Fehlerberichte abholen“) |
| Einstellungen | `environment` in `docker-compose.yml`: `STAMMTISCH_ORIGIN` (Herkunft der Seite, daraus die Passkey-RP-Id), `STAMMTISCH_APP_ORIGIN` (Herkunft der iOS-App, ADR 0026), `STAMMTISCH_DB`, `STAMMTISCH_LISTEN`, optional `STAMMTISCH_BACKUP_DIR` |

Einmalig ist nichts zu tun: `up.sh` legt `data/` selbst an. Secrets braucht der Dienst keine (Sitzungen sind Zufallstokens in der Datenbank).

**Sicherung.** Die Datenbank ist der einzige Zustand auf dem Server. Der Dienst sichert sie selbst: Einmal am Tag (geprüft wird stündlich, das Datum kommt von der Uhr des Servers, UTC) schreibt er mit `VACUUM INTO` eine konsistente Kopie nach `/opt/skat/data/backup/stammtisch-JJJJMMTT.db`, im laufenden Betrieb und ohne `sqlite3` auf dem Server. Die letzten sieben Kopien bleiben, ältere löscht er. Das Log zeigt `backup written`. Ein anderer Ordner geht mit `STAMMTISCH_BACKUP_DIR`, `off` schaltet die Sicherung ab.

Die Kopien liegen auf demselben Server. Gegen den Verlust des Servers gehören sie ab und zu woandershin, etwa vom eigenen Rechner aus:

```
rsync -az deploy@62.83.23.90:/opt/skat/data/backup/ ./stammtisch-backup/
```

Eine Kopie von Hand, sofort: Dienst kurz anhalten und die Datei kopieren (die Seite läuft weiter, nur `/api` ist so lange weg):

```
cd /opt/skat && docker compose stop api && cp data/stammtisch.db "backup-$(date +%F).db" && docker compose start api
```

Zurückspielen: Dienst anhalten, die Kopie nach `data/stammtisch.db` kopieren (alte `-wal`/`-shm` löschen), Dienst starten.

**Schema.** Die Tabellen entstehen aus einer Liste von Migrationen; `PRAGMA user_version` zählt, wie viele eine Datei schon hat. Ein neuer Dienst ergänzt beim Start die fehlenden in einer Transaktion. Eine Datei aus einem neueren Dienst (Rückkehr zu einem alten Binary) fasst er nicht an und startet nicht; dann das neuere Binary wieder ausrollen oder eine Sicherung von vorher zurückspielen.

**Grenzen.** Die Datenbank wächst höchstens auf 2 GiB, danach antwortet jedes Schreiben 507 und die App behält den Fortschritt im Browser. Höchstens 5 neue Konten je Adresse (IPv6: je /64) und Stunde.

**Domain wechseln.** Passkeys gelten nur für die Domain, auf der sie angelegt wurden. Nach einem Umzug `STAMMTISCH_ORIGIN` anpassen; Spieler melden sich einmal mit dem Wiederherstellungscode an und legen einen neuen Passkey an (ADR 0021).

**Lokal entwickeln.** `cargo run -p stammtisch_server` (lauscht auf `127.0.0.1:8787`, Datenbank `stammtisch.db` im aktuellen Ordner, erlaubt `http://localhost:5173` und `:4173`), dazu `npm run dev` oder `npm run preview` in `web/`: Vite leitet `/api` dorthin.

## Fehlerberichte abholen (ADR 0027)

Spieler melden Fehler in der App („Fehler melden“ im Menü); der Konto-Dienst legt sie in der Tabelle `reports` ab (Migration 4), als undurchsichtigen Text mit Zeit, Plattform, Version und einer Nummer wie `B-7K3Q`. Es gibt keinen Lesezugriff über die API. Abgeholt wird über den Workflow `.github/workflows/bug-reports.yml` (nur von Hand), mit denselben Secrets wie der Deploy:

```
gh workflow run bug-reports.yml                    # die letzten 7 Tage
gh workflow run bug-reports.yml -f since=24h       # oder 2026-11-12, 2026-11-12T08:00 (UTC)
gh workflow run bug-reports.yml -f id=B-7K3Q       # genau einer
gh run list --workflow bug-reports.yml --limit 1   # Lauf finden
gh run view <run-id> --log                         # Kurzfassung je Bericht (Text, Zustand, Gerät, letzte Einträge)
gh run download <run-id> -n bug-reports            # vollständig: reports.json (7 Tage aufbewahrt)
```

Der Workflow ruft auf dem Server im laufenden Container `stammtisch-server reports [--since …] [--id …]` auf; das Unterkommando öffnet die Datenbank nur lesend und schreibt JSON auf stdout (`report` ist der Text der App, siehe `web/src/report.ts`). Von Hand auf dem Server:

```
cd /opt/skat && docker compose exec -T api /app/stammtisch-server reports --since 7d
```

Grenzen: 256 KiB je Bericht, 5 je Adresse und Stunde, zusammen höchstens 256 MiB (sonst 507). Berichte älter als 90 Tage löscht der Dienst stündlich, die eines Kontos mit dem Konto.

## Fehlersuche

```
cd /opt/skat && docker compose ps
cd /opt/skat && docker compose logs --tail 50
docker network inspect stammtisch-proxy --format '{{range .Containers}}{{.Name}} {{end}}'
```

Der letzte Befehl muss `stammtisch-web-1` und `haushalt-prod-caddy-1` zeigen. Fehlt der Caddy, ist der Haushalt-PR noch nicht ausgerollt.

Konto-Dienst (ändernde Anfragen von Hand brauchen `-H 'Origin: https://one-more-skat.de'`, sonst 403):

```
cd /opt/skat && docker compose logs --tail 50 api
cd /opt/skat && docker compose exec -T api wget -qO- http://127.0.0.1:8080/api/health    # "ok"
curl -s https://one-more-skat.de/api/health                                                # "ok" von außen
curl -sI https://one-more-skat.de/ | grep -i content-security                              # die CSP ist da
curl -sI https://one-more-skat.de/.well-known/apple-app-site-association                  # 200 JSON mit APPLE_TEAM_ID, sonst 404
ls -l /opt/skat/data/backup/                                                                # die täglichen Sicherungen
sh /opt/skat/up.sh                                                                          # alles neu starten
```
