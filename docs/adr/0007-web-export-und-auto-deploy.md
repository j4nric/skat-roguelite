# 0007: Web-Export auf dem vServer, Auto-Deploy über GitHub Actions

Datum: 2026-10-31 · Status: Angenommen · Plattform-Teil ersetzt durch 0009 (Web-App statt Godot-Web-Export)

## Kontext

Das Spiel soll auf einem vServer laufen und jede Änderung soll die Server-Instanz automatisch neu bauen. Ein Godot-Spiel läuft auf einem Server nur sinnvoll als Web-Export, der vom Browser aus gespielt wird.

## Entscheidung

- Die Zielplattform für den Server ist der **Godot-Web-Export** (HTML5/WebAssembly). Desktop-Exporte bleiben möglich, sind aber nicht Teil des Deploys.
- Jeder Push auf `main` löst den Workflow `deploy.yml` aus: Tests, Simulator-Baseline, Zusammenstellen von `site/`, `rsync` per SSH auf den Server, `docker compose up -d`.
- Auf dem Server läuft nur Caddy in Docker und liefert `site/` aus. Keine Build-Werkzeuge auf dem Server.
- Bis M3 ist `site/` ein Platzhalter aus `web/` plus Simulator-Baseline, damit die Pipeline von Anfang an durchläuft.

## Alternativen

- Build auf dem Server (Webhook, Git-Pull, Build): braucht Rust und Godot auf dem Server und macht den Server zum zweiten Build-System.
- Container-Registry und Image-Pull: sauberer für Backend-Dienste, aber für eine statische Seite Overhead ohne Nutzen. Kann später kommen, wenn ein Backend dazukommt (Highscores, Seeds teilen).

## Folgen

- Das Godot-Projekt muss von Anfang an für Web exportierbar bleiben: keine Plugins ohne Web-Unterstützung, Threads nur mit Cross-Origin-Isolation (Caddy setzt die Header).
- Der Nutzer muss einmalig Server, SSH-Schlüssel und vier GitHub-Secrets einrichten (siehe `deploy/README.md`). Bis dahin überspringt der Workflow den Server-Schritt mit Warnung.
