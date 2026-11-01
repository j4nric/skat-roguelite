// All texts of the web app (German only, see DESIGN 16). Game terms use the
// German deck (ADR 0010, GLOSSARY "Anzeige im deutschen Blatt"). Placeholders
// in braces are filled by `t()` in ./index.ts.

export const de = {
  app: {
    title: 'Stammtisch',
    tagline: 'Skat als Roguelike',
    build: 'Build {info}',
    loading: 'Der Tisch wird gedeckt …',
    loadFailed: 'Das Spiel konnte nicht geladen werden. Bitte Seite neu laden.',
    actionFailed: 'Das ging nicht: {reason}',
    errors: {
      WrongPhase: 'Das geht gerade nicht.',
      NotEnoughCoins: 'Dafür reichen die Groschen nicht.',
      NoRedealLeft: 'In dieser Serie wurde schon geschoben.',
      MustFollowSuit: 'Du musst bedienen.',
      UnknownCard: 'Diese Karte gibt es nicht.',
      default: 'Unbekannter Fehler.',
    } as Record<string, string>,
    close: 'Schließen',
    hurry: 'Tippen zum Beschleunigen',
  },

  start: {
    newRun: 'Neuer Run',
    resume: 'Fortsetzen',
    resumeInfo: 'Abschnitt {section}, Serie {series} · Konto {score} / {target}',
    resumeFailed: 'Der gespeicherte Run passt nicht mehr zu dieser Version und wurde verworfen.',
    seedLabel: 'Seed (leer lassen für Zufall)',
    seedPlaceholder: 'zufällig',
    seedInvalid: 'Der Seed muss eine ganze Zahl sein.',
    record: 'Runs: {runs} · gewonnen: {won} · weitester Abschnitt: {best}',
    noRecord: 'Noch kein Run gespielt.',
    howTo:
      'Du bist immer Alleinspieler. Erreiche in jeder Serie den Zielwert, kaufe am Tresen Hausregeln und verändere das Blatt, mit dem alle drei spielen.',
    sign: 'Gaststätte',
    since: 'Skat seit 1813',
    credits: 'Abspann',
    settings: 'Einstellungen',
  },

  header: {
    section: 'Abschnitt {section}/{sections}',
    series: 'Serie {series}/3',
    bossSeries: 'Boss: {name}',
    account: 'Konto',
    target: 'Ziel {target}',
    games: 'Spiele',
    gamesLeft: '{left} von {limit}',
    passes: 'Pässe',
    coins: 'Groschen',
    seed: 'Seed {seed}',
    menu: 'Menü',
    where: 'Abschnitt {section} · Serie {series}',
    whereBoss: 'Abschnitt {section} · Boss',
    overview: 'Kreidetafel öffnen',
    tourHint: 'Kreidetafel',
    gamesAria: '{left} von {limit} Spielen übrig',
    passesAria: '{left} Pässe übrig',
  },

  suits: {
    Clubs: 'Eichel',
    Spades: 'Grün',
    Hearts: 'Rot',
    Diamonds: 'Schellen',
  },

  ranks: {
    '7': 'Sieben',
    '8': 'Acht',
    '9': 'Neun',
    T: 'Zehn',
    J: 'Unter',
    Q: 'Ober',
    K: 'König',
    A: 'Daus',
  },

  /** Corner index of a card. */
  rankShort: {
    '7': '7',
    '8': '8',
    '9': '9',
    T: '10',
    J: 'U',
    Q: 'O',
    K: 'K',
    A: 'D',
  },

  card: {
    name: '{suit}-{rank}',
    blank: 'Blanko',
    back: 'Verdeckte Karte',
    fromSkat: 'Skat',
    mods: 'Gezinkt: {mods}',
  },

  /** Short badge per card modification id. */
  modShort: {
    fat: 'F',
    marked: 'G',
    stamp: 'St',
    last_trick: 'Sp',
    transparent: 'D',
  } as Record<string, string>,

  games: {
    Grand: 'Grand',
    Null: 'Null',
  },

  risks: {
    Normal: 'Normal',
    Hand: 'Hand',
    SchneiderDeclared: 'Schneider angesagt',
    SchwarzDeclared: 'Schwarz angesagt',
    Ouvert: 'Ouvert',
  },

  risksShort: {
    Normal: 'Normal',
    Hand: 'Hand',
    SchneiderDeclared: 'Schneider',
    SchwarzDeclared: 'Schwarz',
    Ouvert: 'Ouvert',
  },

  riskHints: {
    Normal: 'Skat aufnehmen, zwei Karten drücken.',
    Hand: 'Skat bleibt liegen. Eine Stufe mehr.',
    SchneiderDeclared: 'Hand, dazu Schneider angesagt. Schaffst du ihn nicht, ist das Spiel verloren.',
    SchwarzDeclared: 'Hand, dazu Schwarz angesagt: alle Stiche oder verloren.',
    Ouvert: 'Wie Schwarz angesagt, aber mit offenen Karten.',
  },

  positions: {
    Forehand: 'Vorhand',
    Middlehand: 'Mittelhand',
    Rearhand: 'Hinterhand',
  },

  seats: ['Du', 'Gegner links', 'Gegner rechts'],
  seatsShort: ['Du', 'Links', 'Rechts'],

  table: {
    declareTitle: 'Ansage',
    gameType: 'Spielart',
    risk: 'Risikostufe',
    declare: 'Ansagen',
    pass: 'Einpassen ({left})',
    redraw: 'Schieben ({left})',
    handRequired: 'Der Boss verlangt ein Handspiel.',
    discardHint: 'Tippe zwei Karten an. Gedrückte Augen zählen für dich.',
    discard: 'Drücken',
    yourTurn: 'Du bist dran',
    trickTo: 'Stich an {name} ({points} Augen)',
    trickToYou: 'Dein Stich ({points} Augen)',
    tricks: 'Stiche {own} : {theirs}',
    skat: 'Gedrückt',
    declaration: '{game} · {risk}',
    cards: '{count} Karten',
    kontraBadge: 'Kontra',
    reBadge: 'Re',
    holds: 'hält',
    game: 'Spiel {number}',
    tricksOwn: 'Deine Stiche',
    you: 'Du',
  },

  kontra: {
    title: 'Kontra!',
    text: '{name} sagt Kontra. Der Spielwert verdoppelt sich.',
    question: 'Sagst du Re? Dann zählt das Spiel vierfach.',
    re: 'Re sagen (×4)',
    noRe: 'Kein Re',
  },

  odds: {
    line: 'Jede Karte landet zu {pct} bei dir oder im Skat.',
    detail: 'Hand {hand} · Skat {skat} · Gegner {defenders}',
    talon: ' · Talon {talon}',
  },

  toast: {
    bossAnnounced: 'Neuer Skatabend. Der Boss heute: {name}.',
    passed: 'Eingepasst. Noch {left} Pässe.',
    redealt: 'Neu gemischt und gegeben.',
    conceded: 'Die Gegner schenken dir den Rest.',
    kontra: '{name}: Kontra!',
    re: 'Re!',
    consumableUsed: '{name} benutzt.',
    passesAdded: 'Ein Pass mehr: jetzt {left}.',
    bought: '{name} gekauft.',
    sold: '{name} verkauft (+{price} Gr.).',
    rerolled: 'Neue Auslage.',
  },

  score: {
    won: 'Gewonnen!',
    lost: 'Verloren',
    points: 'Augen {own} : {theirs}',
    tricks: 'Stiche {own} : {theirs}',
    baseOf: 'Grundwert {game}',
    nullValue: 'Nullspiel',
    with: 'mit {count}',
    without: 'ohne {count}',
    game: 'Spiel',
    hand: 'Hand',
    schneider: 'Schneider',
    schneiderDeclared: 'Schneider angesagt',
    schwarz: 'Schwarz',
    schwarzDeclared: 'Schwarz angesagt',
    ouvert: 'Ouvert',
    plusLevels: '+{n} Stufe',
    plusLevelsMany: '+{n} Stufen',
    minusLevels: '{n} Stufen',
    baseAdd: 'Grundwert +{n}',
    baseSub: 'Grundwert {n}',
    baseMul: 'Grundwert ×{n}',
    valueMul: 'Spielwert ×{n}',
    pointsAdd: 'Augen +{n}',
    pointsSub: 'Augen {n}',
    threshold: 'Gewonnen ab {n} Augen',
    schneiderBy: 'zählt als Schneider',
    setTo: '= {n}',
    kontra: 'Kontra',
    re: 'Re',
    cardMod: '{mod} ({card})',
    formula: '{base} × {levels}',
    formulaMul: '{base} × {levels} × {mul}',
    value: 'Spielwert',
    result: 'Ergebnis',
    lostDouble: 'Verlorene Spiele zählen doppelt.',
    account: 'Konto {score} / {target}',
    continue: 'Weiter',
    skip: 'Überspringen',
    tapToSkip: 'Tippen zum Überspringen',
    boxBase: 'Grundwert',
    boxLevels: 'Stufen',
    boxMul: 'Faktor',
    accountLabel: 'Konto',
    targetLabel: 'Ziel {target}',
    seriesWon: 'Serie gewonnen!',
    payoutBase: 'Grundbetrag',
    payoutGames: 'Übrige Spiele',
    payoutInterest: 'Zinsen',
  },

  shop: {
    title: 'Am Tresen',
    subtitle: 'Kaufe Hausregeln und Gegenstände für die nächste Serie.',
    relics: 'Hausregeln und Figuren',
    consumables: 'Gegenstände',
    consumableKind: 'Gegenstand',
    owned: 'Deine Relikte ({count}/{slots})',
    ownedItems: 'Deine Gegenstände ({count}/{slots})',
    empty: 'Nichts.',
    soldOut: 'Verkauft',
    buy: 'Kaufen · {price} Gr.',
    sell: 'Verkaufen +{price}',
    use: 'Benutzen',
    reroll: 'Neuauslage · {cost} Gr.',
    leave: 'Weiter',
    pickTarget: 'Wähle eine Karte',
    pickHint: '{name}: {text}',
    cancel: 'Abbrechen',
    wirtSays: 'Na, was darf’s sein?',
    next: 'Als Nächstes',
    nextSeries: 'Abschnitt {section} · Serie {series}',
    nextBoss: 'Abschnitt {section} · Boss-Serie',
    nextTarget: 'Ziel {target}',
    nextGames: '{games} Spiele · {passes} Pässe',
    bossTonight: 'Boss dieses Abends',
    lastSeries: 'Das war die letzte Serie der Tournee.',
    wirtBroke: 'Anschreiben gibt’s hier nicht.',
    price: '{price} Gr.',
    buyShort: 'Kaufen',
    noRelics: 'Noch keine Bierdeckel gesammelt.',
    noItems: 'Nichts in der Tasche.',
    moreAbout: '{name}: ganze Beschreibung',
    moreBelow: 'Weitere Waren weiter unten',
  },

  rarity: {
    Common: 'gewöhnlich',
    Uncommon: 'ungewöhnlich',
    Rare: 'selten',
  },

  relicKind: {
    HouseRule: 'Hausregel',
    Regular: 'Figur',
  },

  boss: {
    title: 'Boss-Serie',
    intro: 'Am Tisch sitzt jetzt:',
    go: 'Los geht’s',
    unknown: '?',
  },

  end: {
    won: 'Tournee gewonnen!',
    wonText: 'Der ganze Stammtisch zieht den Hut.',
    lost: 'Run verloren',
    lostText: 'Ausgeschieden in Abschnitt {section}, Serie {series}.',
    gamesWon: 'Spiele gewonnen',
    gamesLost: 'Spiele verloren',
    gamesPassed: 'Eingepasst',
    seriesWon: 'Serien gewonnen',
    bestGame: 'Bestes Spiel',
    coins: 'Groschen',
    seed: 'Seed',
    newRun: 'Neuer Run',
    toStart: 'Zum Start',
    overview: 'Kreidetafel',
  },

  menu: {
    tempo: 'Tempo',
    tempos: { slow: 'Gemütlich', normal: 'Normal', fast: 'Flott' },
    sound: 'Ton',
    soundOn: 'An',
    soundOff: 'Aus',
    overview: 'Kreidetafel',
    credits: 'Abspann',
    reducedMotion: 'Dein Gerät wünscht weniger Bewegung. Animationen sind deshalb kurz.',
  },

  overview: {
    title: 'Kreidetafel',
    tour: 'Die Tournee',
    section: 'Abschnitt {n}',
    columns: ['Serie 1', 'Serie 2', 'Boss'],
    current: '{score} / {target}',
    done: 'geschafft',
    lost: 'ausgeschieden',
    thisRun: 'Diese Tournee',
    allRuns: 'Alle Runs',
    runs: 'Runs',
    runsWon: 'Gewonnen',
    bestSection: 'Weitester Abschnitt',
    deck: 'Dein Blatt ({count} Karten)',
    deckHint: 'Dieses Blatt teilen sich alle drei am Tisch.',
  },

  credits: {
    title: 'Abspann',
    intro: 'Stammtisch ist ein Projekt mit Liebe zum Skat. Diese Quellen stecken drin:',
    entries: [
      {
        what: 'Schrift „Grenze Gotisch“',
        who: 'Omnibus-Type (The Grenze Gotisch Project Authors)',
        license: 'SIL Open Font License 1.1',
      },
      {
        what: 'Schrift „Alegreya Sans“',
        who: 'Juan Pablo del Peral, HT Fonts (Huerta Tipográfica)',
        license: 'SIL Open Font License 1.1',
      },
      {
        what: 'Kartenbild (deutsches Blatt), Figuren, Tresen',
        who: 'Selbst gezeichnet als SVG für dieses Projekt',
        license: 'Teil des Projekts',
      },
      {
        what: 'Klänge (Karten, Stich, Kreide, Gläser, Groschen, Klopfen, Akkordeon)',
        who: 'Selbst synthetisiert für dieses Projekt (scripts/make-sounds.mjs)',
        license: 'Teil des Projekts',
      },
    ],
    thanks: 'Gut Blatt!',
  },

  flags: {
    bock: 'Bockrunde: das nächste Spiel zählt doppelt',
  } as Record<string, string>,

  counters: {
    tour: 'Tournee: +{value} Grundwert',
  } as Record<string, string>,

  /**
   * Display names and texts that depend on the French deck in the content
   * files get German-deck versions here (ADR 0010). Anything not listed
   * shows the name and text from the core.
   */
  content: {
    relics: {
      old_lady: {
        name: 'Alter Ober',
        text: 'Jeder Ober zählt 5 Augen. Auch bei den Gegnern.',
      },
    },
    card_mods: {},
    bosses: {
      grandma: {
        text: 'Die Oma hat alle Unter eingesteckt. In dieser Serie sind sie Blankos.',
      },
    },
    consumables: {},
  } as Record<'relics' | 'card_mods' | 'bosses' | 'consumables', Record<string, { name?: string; text?: string }>>,
} as const;

export type Texts = typeof de;
