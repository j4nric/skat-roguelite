// The lines of the store pictures and the preview video, German and
// English. "\n" breaks a line. Same order as the scenes in
// make-screenshots.mjs and the cuts in make-preview.mjs.

export const LINES = {
  de: {
    shots: [
      'Echter Skat.\nJeden Abend anders.',
      'Brich die Regeln.\nStapel die Faktoren.',
      'Gezinkte Karten.\nUntote Karten.',
      'Hausregeln und\nSchnäpse vom Wirt.',
      'Bosse mit\neigenen Regeln.',
      'Zieh durch die\nKneipen der Stadt.',
    ],
  },
  en: {
    shots: [
      'Real Skat.\nDifferent every night.',
      'Break the rules.\nStack the multipliers.',
      'Marked cards.\nUndead cards.',
      'House rules and the\nlandlord\'s schnapps.',
      'Bosses with\ntheir own rules.',
      'Crawl through\nthe city\'s pubs.',
    ],
  },
};

/** Lines over the preview video (keys used by assemble.py's timeline) and the end card. */
export const VIDEO = {
  de: {
    lines: {
      real: 'Echter Skat.',
      nights: 'Jeden Abend anders.',
      rules: 'Brich die Regeln.',
      undead: 'Untote Karten.',
      doppel: 'Doppelgänger.',
      cursed: 'Verfluchte Karten.',
      tresen: 'Hausregeln. Schnäpse.\nGezinkte Karten.',
      bosses: 'Bosse mit\neigenen Regeln.',
      city: 'Zieh durch die Stadt.',
    },
    end: { kicker: 'Gaststätte', title: "Noch 'ne\nRunde?", tagline: 'Skat als Roguelike', store: 'Jetzt im App Store' },
  },
  en: {
    lines: {
      real: 'Real Skat.',
      nights: 'Different every night.',
      rules: 'Break the rules.',
      undead: 'Undead cards.',
      doppel: 'Doppelgängers.',
      cursed: 'Cursed cards.',
      tresen: 'House rules. Schnapps.\nMarked cards.',
      bosses: 'Bosses with\ntheir own rules.',
      city: 'Crawl the city.',
    },
    end: { kicker: 'Pub', title: 'One more\nround?', tagline: 'Skat as a roguelike', store: 'Now on the App Store' },
  },
};
