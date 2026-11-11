# 0024: Eigener Eintrag im vorderen Caddy

Datum: 2026-11-11 · Status: Angenommen · Ergänzt 0008

## Kontext

Nach ADR 0008 hängt das Spiel hinter dem Caddy der Haushalt-App, der die Ports 80 und 443 des Servers belegt. Jede Domain und jedes Netzwerk stand dort im Repo `j4nric/haushalt-app`. Eine neue Domain (one-more-skat.de für die iOS-App, Cockpit q40) hieß deshalb: PR im fremden Repo, dessen CI abwarten, mergen. Dasselbe galt für jede andere App auf dem Server. Der Nutzer wollte das ändern.

## Entscheidung

- Der vordere Caddy liest zusätzlich jede Datei `*.caddy` aus dem gemeinsamen Docker-Volume `caddy-sites` (`import /etc/caddy/sites/*.caddy`) und trägt das Label `edge.caddy=true`. Alle Apps erreicht er über das gemeinsame Netzwerk `edge`. Das ist die letzte Änderung dafür im Haushalt-Repo.
- Unser Eintrag steht in `deploy/site.caddy`. `deploy/up.sh` prüft ihn einzeln (`caddy validate`), legt ihn als `stammtisch.caddy` ins Volume, lässt den vorderen Caddy die ganze Konfiguration prüfen und lädt ihn ohne Neustart neu (`caddy reload`). Lehnt er ab, kommt der vorige Eintrag zurück und der Deploy schlägt fehl; der vordere Caddy läuft mit dem alten Stand weiter.
- Der Container `web` hängt im Netzwerk `edge` mit dem Alias `stammtisch-web`, bis zum Umzug auch weiter in `stammtisch-proxy`.

## Alternativen

- Ein eigenes Repo nur für den vorderen Caddy, aus dem die Haushalt-App auszieht: am saubersten, aber ein Umzug aller Seiten des Servers mit neuen Zertifikaten. Später möglich, der Mechanismus bleibt derselbe.
- caddy-docker-proxy (Einträge als Labels an den Containern): braucht ein anderes Caddy-Image vorne und ändert alle bestehenden Einträge.
- Weiter alles im Haushalt-Repo: genau das, was der Nutzer loswerden wollte.

## Folgen

- Domains des Spiels ändern sich mit einem PR in diesem Repo.
- Reihenfolge beim ersten Mal: erst dieses Repo ausrollen (legt Netzwerk, Volume und Eintrag an), dann den Haushalt-PR, der den alten Block entfernt und den Import einschaltet. Andersherum ist das Spiel bis zum nächsten Deploy hier nicht erreichbar.
- Jeder Name darf auf dem ganzen Server nur einmal vorkommen; eine doppelte Domain lässt den Reload scheitern, der vordere Caddy bleibt aber am Laufen.
- Der Deploy-Nutzer braucht keine neuen Rechte: Er bedient Docker schon.
- Andere Apps können nach demselben Muster umziehen; ihre Blöcke im Haushalt-Caddy laufen bis dahin weiter.
