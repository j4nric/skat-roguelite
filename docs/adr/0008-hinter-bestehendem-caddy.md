# 0008: Hinter dem bestehenden Caddy der Haushalt-App

Datum: 2026-10-31 · Status: Angenommen · Ergänzt 0007

## Kontext

Auf dem vServer laufen bereits die Haushalt-App, der Sync-Server von „Vom Samen zum Wald“ und die Tee-Etiketten. Ports 80 und 443 gehören dem Caddy der Haushalt-App. Dessen Caddyfile ist laut Kommentar die einzige Proxy-Konfiguration des Servers und wird bei jedem Haushalt-Deploy aus dem Repo überschrieben. Der ursprüngliche Plan aus ADR 0007, einen eigenen Caddy auf 80/443 zu starten, ist damit nicht möglich.

## Entscheidung

- Unser Container veröffentlicht keine Ports. Er hängt im eigenen Netzwerk `stammtisch-proxy` mit dem Alias `stammtisch-web`.
- Der Haushalt-Caddy hängt zusätzlich in diesem Netzwerk und hat einen Block für `stammtisch-62-83-23-90.sslip.io`. Beides steht im Repo `j4nric/haushalt-app`.
- Domain vorerst über sslip.io, kostenlos und ohne Registrierung.

Das ist dasselbe Muster wie bei `tee-proxy` und `vszw-proxy`.

## Alternativen

- Eigener Caddy auf anderem Port: kein HTTPS über Standardports, für einen Web-Export mit Cross-Origin-Isolation unpraktisch.
- Block von Hand in die Caddyfile auf dem Server: geht beim nächsten Haushalt-Deploy verloren.

## Folgen

- Eine Änderung an Domain oder Netzwerk braucht einen PR im Haushalt-Repo.
- Reihenfolge bei der Einrichtung: erst Netzwerk auf dem Server, dann Haushalt-Deploy, sonst scheitert dieser.
- Header wie Cache-Control setzt unser Caddy. Der vordere Caddy reicht sie durch.
