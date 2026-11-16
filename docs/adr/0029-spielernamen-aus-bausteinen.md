# 0029: Spielernamen aus Bausteinen statt freiem Text

Datum: 2026-11-14 · Status: Angenommen

## Kontext

Seit ADR 0021 setzt ein angemeldeter Spieler einen frei gewählten Namen (1 bis 24 Zeichen), der auf der Tafel des Tages und der Tafel der Woche für alle sichtbar steht. Apple wertet solche Namen in der App-Prüfung (Richtlinie 1.2, „User-Generated Content“) als Nutzerinhalte. Dafür verlangt Apple einen Filter gegen anstößige Inhalte, eine Möglichkeit, Inhalte und Spieler zu melden, eine Möglichkeit, Spieler zu blockieren, und eine erreichbare Ansprechperson, die gemeldete Inhalte zeitnah entfernt. Für ein Kartenspiel mit einer kleinen Tafel wäre das unverhältnismäßig viel Moderation, und ein Wortfilter für zwei Sprachen bleibt immer lückenhaft.

Der Nutzer hat am 14.11.2026 entschieden: Statt freiem Text setzt sich der Name aus festen Bausteinen zusammen, überall (Browser und App).

## Entscheidung

Ein Name besteht aus einer **Eigenschaft** (Adjektiv), einem **Wer** (Substantiv) und einer freiwilligen **Nummer** von 1 bis 99, etwa „Durstiger Ober 7“, „Schlaue Wirtin“, „Edles Daus“; englisch „Thirsty Ober 7“, „Clever Landlady“, „Noble Ace“. Je Liste 28 Einträge, also 784 Namen, mit Nummer rund 78 000.

- **Gespeichert und übertragen werden Ids, kein Text**: `a<Eigenschaft>.n<Wer>` oder `a<Eigenschaft>.n<Wer>.<Nummer>` (`a3.n12`, `a3.n12.7`), Indizes ab 0 ohne führende Nullen. Jeder liest einen Namen in seiner eigenen Sprache.
- **Die Wörter stehen nur in der Web-App** (`playerNames` in `web/src/i18n/de.ts` und `en.ts`). Der Konto-Dienst kennt nur die Form und die Länge der Listen (`server/src/names.rs`: `ADJECTIVES`, `NOUNS`, `MAX_NUMBER`) und nimmt bei `POST /api/register/start` und `PUT /api/me/name` nur gültige Ids oder einen leeren Namen (entfernen) an, sonst 400 `name`. `npm run typecheck` prüft, dass die Listen beider Sprachen so lang sind, wie der Server erlaubt.
- **Die Listen wachsen nur am Ende.** Ein Eintrag wird nie umsortiert, umbenannt oder entfernt, sonst änderte sich der Name gespeicherter Spieler. Ein neuer Baustein heißt: Wort in beiden Sprachen anhängen und die Zahl im Server erhöhen.
- **Deutsche Grammatik**: Ohne Artikel trägt das Adjektiv die starke Endung nach dem Geschlecht des Substantivs (Durstiger Ober, Durstige Wirtin, Durstiges Blatt). Jedes Adjektiv steht deshalb mit seinen drei Formen in der Liste („Durstiger|Durstige|Durstiges“), jedes Substantiv mit seinem Geschlecht („Wirtin|f“). Ausdrücklich ausgeschriebene Formen statt Endungsregeln, weil Wörter wie „edel“ (edler, edle, edles) sonst Sonderfälle bräuchten. Substantive aller drei Geschlechter bleiben möglich, die Auswahl wird dadurch reicher. Englisch kennt kein Geschlecht: eine Form, keine Markierung.
- **Wortwahl**: Kneipe und Skat, freundlich. Keine Körper-, Schimpf- oder Rauschwörter: Die Eigenschaften sind freundliche Charakterzüge, die Substantive Karten, Rollen, Dinge und Tiere der Kneipe, so dass auch jede Kombination harmlos bleibt (die Paare sind durchgesehen; „Alter Fuchs“ ist das Derbste). „Ober“ steht hier für den Kellner („Herr Ober!“), die Karte heißt im Spiel Oberin (ADR 0014).
- **Die Nummer** macht gleiche Namen auf einer Tafel seltener (bei 20 Spielern ohne Nummer teilen sich mit gut 20 % Wahrscheinlichkeit zwei denselben Namen). Sie ist freiwillig, Voreinstellung „ohne“.
- **Oberfläche**: Im Konto-Blatt drei Auswahllisten mit Beschriftung (Eigenschaft, Wer, Nummer), alphabetisch in der aktiven Sprache, die Eigenschaft in der Form für das gewählte Wer, ein Knopf „Würfeln“ für einen zufälligen Namen und eine Vorschau „So heißt du: …“. „Konto anlegen“ bietet einen gewürfelten Namen an. „Ohne Namen“ entfernt ihn. Überall, wo ein Name steht (Tafeln, „Angemeldet als …“, „Konto: …“ im Menü, das Umzugsblatt), wird aus den Ids der Text der aktiven Sprache; was keine gültige Id ist, erscheint als „Namenlos“.

## Alternativen

- **Freier Text mit Wortfilter, Melden und Blockieren**: erfüllt 1.2, braucht aber Moderation, eine Melde-Oberfläche, Sperrlisten auf dem Server und jemanden, der Meldungen bearbeitet. Ein Filter in zwei Sprachen bleibt lückenhaft.
- **Gar keine Namen auf der Tafel** (nur „Spieler 1“, „Du“): kein Nutzerinhalt, aber die Tafel verliert ihren Reiz, und der eigene Platz ist schwer wiederzufinden, wenn man auf einem anderen Gerät schaut.
- **Die Wörter auch auf dem Server**: Der Server könnte Text ausliefern, müsste dann aber beide Sprachen kennen und bei jedem neuen Wort mit ausgeliefert werden. Ids halten den Server sprachfrei; er braucht nur die Längen der Listen.
- **Nur Substantive eines Geschlechts** (etwa nur männliche): einfachere Grammatik, aber die Liste würde einseitig (keine Wirtin, keine Skatschwester, kein Daus).

## Folgen

- Es gibt keine Nutzerinhalte mehr: Kein Spieler kann Text eingeben, den andere sehen. Filter, Melden, Blockieren und Moderation entfallen. Im Altersfragebogen von App Store Connect sind Nutzerinhalte damit „nein“ (`docs/ios.md`).
- **Bestehende Namen werden gelöscht**: Migration 5 des Konto-Dienstes (`server/src/db.rs`) setzt jeden gespeicherten Namen, der keine gültige Id ist, auf leer. Diese Spieler stehen als „Namenlos“ auf der Tafel, bis sie einen Namen wählen; Konto, Spielstand und Ergebnisse bleiben.
- Der Passkey heißt im Passwort-Manager immer „Stammtisch-Spieler“, weil der Server die Wörter eines Namens nicht kennt (bisher stand dort der Name).
- Eine alte App-Version, die noch freien Text sendet, bekommt 400 `name`; die Fehlermeldung bittet, die Seite neu zu laden. Spielstand, Wiederherstellung und Umzug (ADR 0025) tragen den Namen nicht und sind nicht betroffen; die Sync-Notiz im Browser merkt sich die Id und zeigt sie wie überall.
- Ändert ADR 0021 im Punkt „Name“ (dort 1 bis 24 freie Zeichen).
