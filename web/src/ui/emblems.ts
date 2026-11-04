// Emblems (style "R1 Bierdeckel", v2 "Deckel v2"): every relic, consumable
// and boss is drawn as a small object with its own 16 × 16 pixel motif.
// - Relics are beer mats whose shape and paper follow their category
//   (RelicGroup): Grundwert round, Multiplikator a sunburst, Kontrolle a
//   rounded square, Tresen und Tour an inn sign shield.
// - Schnäpse are no mats but consumables at first sight: a bottle, a label
//   or a shot glass (SchnapsStyle). Other items keep an octagonal mat.
// - Bosses keep their torn dark mat with a red dashed ring.
// - The rarity has one ink per tier on every form (RARITY_INK), drawn as a
//   rim band, a corner gem or a ring with pips (RarityStyle).
// Each emblem is tilted a little, by its id, and casts a hard shadow.
//
// Sprites are rows of palette keys ('.' is empty); a dark outline is drawn
// around every motif. Rows may be ragged: the motif is centred by its
// bounding box. Relics and consumables have up to three motif variants,
// the one shown is picked in CHOSEN; bosses have one. Unknown ids get a
// generic motif of their kind. Pure presentation, no game logic; this
// module has no runtime imports so scripts can load it directly.

import type { ConsumableCategory, Rarity, RelicGroup, RelicKind } from '../types';

export type { RelicGroup };

/** One motif: German label (for the art vote only), rows, palette overrides. */
export interface Sprite {
  label: string;
  art: string;
  pal?: Record<string, string>;
}

/** The common palette: neutrals of the app plus the four suit inks. */
const PALETTE: Record<string, string> = {
  k: '#1b1e21', // ink
  w: '#ecebe4', // cream
  h: '#e8f4f8', // glass highlight
  g: '#a4adb3', // grey
  G: '#4f6272', // slate
  d: '#2a2f34', // dark panel
  o: '#d1773f', // orange
  O: '#8a4a22', // dark orange
  l: '#f1b48a', // light orange
  m: '#8fc7a8', // mint
  M: '#3f6b55', // dark mint
  b: '#8a5a2b', // brown (wood, leather)
  J: '#4f86c6', // Eichel (jeans blue)
  L: '#f5d52a', // Schellen (lemon)
  e: '#2e7d4f', // Grün
  r: '#bd3b32', // Rot
  R: '#e5776d', // light red
  y: '#b8840f', // ochre gold
  Y: '#e3b24c', // light gold
  p: '#d98e9a', // pink
  s: '#e0b48a', // skin
  n: '#4a2c12', // dark brown
  v: '#5e3796', // violet
  c: '#9fc8d6', // glass
  a: '#e8a417', // beer
  f: '#f6e7b0', // pale spirit
};

const OUTLINE = '#1b1e21';

/** Bright blue of Blue Curaçao (u light, U dark), only for its motifs. */
const CURACAO = { u: '#1fa8f0', U: '#0d6fb5' };

const sp = (label: string, art: string, pal?: Record<string, string>): Sprite => (pal ? { label, art, pal } : { label, art });

/**
 * Motif variants per relic and consumable id: three for the items of the
 * art vote, one for later ones (shown as variant A).
 */
export const SHOP_ART: Record<string, [Sprite, ...Sprite[]]> = {
  // ---- house rules and Stammtisch figures (core/content/house_rules.ron)
  last_trick_seven: [
    sp('Karte mit Sieben', `
      ....wwwwwwww
      ...wwwwwwwwww
      ...wwrrrrrrww
      ...wwrrrrrrww
      ...wwwwwwrrww
      ...wwwwwrrwww
      ...wwwwwrrwww
      ...wwwwrrwwww
      ...wwwwrrwwww
      ...wwwrrwwwww
      ...wwwrrwwwww
      ...wwwwwwwwww
      ....wwwwwwww`),
    sp('Berggipfel mit Fahne', `
      .......rrr
      .......rr
      .......k
      ......www
      .....wwwww
      ....wwGwGww
      ...GGGGGGGGG
      ..GGGGGGGGGGG
      .GGGGGGGGGGGGG
      GGGGGGGGGGGGGGG`),
    sp('Daumen hoch', `
      .....ss
      ....sss
      ....ss
      ...ssssssss
      rr.ssssssss
      rr.sssssskk
      rr.ssssssss
      rr.sssssskk
      rr.ssssssss
      rr..sssssss`),
  ],
  concede: [
    sp('Geschenk', `
      ....yyy..yyy
      ...y..yyyy..y
      ....yyyyyyyy
      ..rrrrryyrrrrr
      ..rrrrryyrrrrr
      ..kkkkkyykkkkk
      ...rrrryyrrrr
      ...rrrryyrrrr
      ...rrrryyrrrr
      ...rrrryyrrrr
      ...rrrryyrrrr`),
    sp('Weiße Fahne', `
      b
      bwwwww
      bwwwwwwww
      bwwwwwwwww
      bwwwwwwwww
      bwwwwwwww
      b..wwwww
      b
      b
      b
      b
      bb`),
    sp('Schleife', `
      rr.........rr
      rrrr.....rrrr
      rrrrrr.rrrrrr
      rrrrrRRRrrrrr
      rrrrrRRRrrrrr
      rrrrrr.rrrrrr
      rr..rr.rr..rr
      ....rr.rr
      ...rr...rr
      ..rr.....rr
      .rr.......rr`),
  ],
  skat_redraw: [
    sp('Hand schiebt Karten', `
      ........wwwwww
      ........wrrrrww
      ..ssss..wrkkrww
      .ssssssswrrrrww
      .ssssssswrkkrww
      .ssssssswrrrrww
      ..sssssswrrrrww
      ..ssss..wrrrrww
      ........wwwwwww
      ............o
      .....ooooooooo
      ............o`),
    sp('Karte in Fahrt', `
      .......wwwwwww
      .......wrrrrrw
      ..ggg..wrkrkrw
      .......wrrrrrw
      gggg...wrkrkrw
      .......wrrrrrw
      ..ggg..wrkrkrw
      .......wrrrrrw
      .......wwwwwww`),
    sp('Schubkarre mit Karten', `
      ....w.r.e
      ...ww.rr.ee
      .GGGGGGGGGGGG
      ..GGGGGGGGGG
      bbbGGGGGGGG
      ....bb..bbkkk
      .........kgggk
      .........kgkgk
      .........kgggk
      ..........kkk`),
  ],
  virgin: [
    sp('Gänseblümchen', `
      ......www
      ..ww..www..ww
      ..wwwwwwwwwww
      ...wwwyyywww
      .wwwwwyyywwwww
      .wwwwwyyywwwww
      ...wwwwwwwww
      ..wwwwwwwwwww
      ..ww..www..ww
      ......www
      .......e
      .....eee
      .......eee
      .......e`),
    sp('Myrtenkranz', `
      ....eeweee
      ..eeweeeewee
      .ewe......eew
      .ee........ee
      .we........ew
      .ee........ee
      .eew......ewe
      ..eeweeweeee
      ....eeeewe
      .....rr.rr
      ....rr...rr`),
    sp('Einhorn', `
      .........Y
      ........Y
      ......w.Y
      .....wwwwp
      ....wwkwwpp
      ...wwwwwwwpp
      ..wwwwwwwwpp
      .wwwwwwwwwpp
      wpwwwwwwwwp
      .ww..wwwwwp
      .....wwwwwpp
      .....wwwwww`),
  ],
  old_lady: [
    sp('Alter Ober mit Federhut', `
      .......rrr
      ......rr
      ...yyyyyyy
      ..yyyyyyyyy
      ...sssssss
      ...sksssks
      ...sssssss
      ..ggggggggg
      ..gggrrrggg
      ...ggggggg
      ....ggggg
      ..vvvvvvvvv
      .vvvvvyvvvvv`),
    sp('Ober-Karte', `
      wwwwwwwwww
      wwwwwrrwww
      wwwyyyyyww
      wwwwssswww
      wwwwgggwww
      wwvvvvvvvw
      wwvvvyvvvw
      wwvvvvvvvw
      wwwvvwvvww
      wwwwwwwwww`),
    sp('Tablett mit Gläsern (Herr Ober)', `
      .ww..ww..ww
      .aa..aa..aa
      .aa..aa..aa
      ggggggggggg
      .....ss
      ....ssss
      ...wwwww
      ..wwwww`),
  ],
  tour: [
    sp('Koffer mit Aufklebern', `
      ......kkkk
      ......k..k
      ..bbbbbbbbbbbb
      ..bbbbbbbbbbbb
      ..bbrrrbbbbbbb
      ..bbrrrbbbmmbb
      ..bbbbbbbbmmbb
      ..kkkkkkkkkkkk
      ..bbbyybbbbbbb
      ..bbbyybbbbbbb
      ..bbbbbbbbbbbb`),
    sp('Reisebus', `
      .YYYYYYYYYYYYY
      YccYccYccYccYcY
      YccYccYccYccYcY
      YYYYYYYYYYYYYYY
      rrrrrrrrrrrrrrr
      YYYYYYYYYYYYYYY
      YYkkkYYYYYkkkYY
      ..kgk.....kgk
      ...k.......k`),
    sp('Landkarte mit Route', `
      wwwwwfffffwwww
      wwwwwfffffrwrw
      wwwwwfffffwrww
      wwwwwfffffrwrw
      wwwwwffrffwwww
      wwwwwfffffwwww
      wwwwwrffffwwww
      wwwwwfffffwwww
      wwwrwfffffwwww
      wwwwwfffffwwww
      wrwwwfffffwwww`),
  ],
  kibitzer: [
    sp('Kiebitz', `
      .........kk
      ........k
      .......kkkk
      ......kkkkkk
      .....kkwkkkkk
      .....kkkkkkkk
      ....okkkkgggg
      .....kkkggggg
      ......wwwwwgg
      .....wwwwwwww
      .....wwwwwwww
      ......wwwwww
      .......y..y
      ......yy..yy`),
    sp('Augen über dem Blatt', `
      ....nnnnnn
      ...nnnnnnnn
      ...ssssssss
      ...swksswks
      ...ssssssss
      wwwkwwwkwwwkwww
      wrwkwewkwywkwbw
      wwwkwwwkwwwkwww
      wwwkwwwkwwwkwww`),
    sp('Fernglas', `
      ..GGG...GGG
      .GGGGG.GGGGG
      GGcccGGGcccGG
      GchccGGGchccG
      GccccG.GccccG
      .GccG...GccG
      ..GG.....GG`),
  ],
  landlord: [
    sp('Wirt mit Schnurrbart', `
      .....ssssss
      ....ssssssss
      ...nssssssssn
      ...nssksskssn
      ...npsssssspn
      ...nnnnnnnnnn
      ....sssrrsss
      .....ssssss
      ..wwwrrkrrwww
      ..wwwwwwwwwwww
      .wwwwwwwwwwwwww
      .wwwwwwwwwwwwww`),
    sp('Zapfhahn', `
      ..YYYYY
      ...bbb
      ...bbb
      ...bbb
      gggggggggg
      gggggggggg
      ........gg
      .........a
      .........a
      .......cccc
      .......caac
      .......caac
      .......cccc`),
    sp('Preisschild mit Prozent', `
      ...oooooooooooo
      ..ooookkoookooo
      .ooo.okkookoooo
      ooooooooookooooo
      ooooooookoooooo
      .ooooookookkooo
      ..ooookoookkooo
      ...oooooooooooo`),
  ],
  card_counter: [
    sp('Abakus', `
      ..bbbbbbbbbbbb
      ..b..........b
      ..brrrgggggggb
      ..b..........b
      ..bggeeeeggggb
      ..b..........b
      ..byyggggggyyb
      ..b..........b
      ..bgggoooogggb
      ..b..........b
      ..bbbbbbbbbbbb
      ..b..........b`),
    sp('Notizblock mit Summen', `
      .k.k.k.k.k
      wwwwwwwwwww
      wGGGwGGwGGw
      wwwwwwwwwww
      wGGwGGGGwww
      wwwwwwwwwww
      wGGGGwGGGww
      wwwwwwwwwww
      wwwwwwrrrrw
      wwwwwwGGGGw
      wwwwwwwwwww`),
    sp('Taschenrechner', `
      GGGGGGGGG
      GmmmmmmmG
      GmkmkmkmG
      GGGGGGGGG
      GwGwGwGoG
      GGGGGGGGG
      GwGwGwGoG
      GGGGGGGGG
      GwGwGwGoG
      GGGGGGGGG
      GwwwGwGoG
      GGGGGGGGG`),
  ],
  every_bit_counts: [
    sp('Huhn', `
      ..........rr
      .........rrr
      ........wwww
      ........wwkwo
      ........wwwwoo
      ..w.....wwwwr
      ..ww...wwwww
      ..wwwwwwwwwww
      ..wwwgggwwwww
      ...wwgggwwww
      ....wwwwwww
      ......o..o
      .....oo.oo`),
    sp('Küken', `
      ....YYY
      ...YYYYY
      ...YYkYYo
      ...YYYYYoo
      .YYYYYYY
      YYYYYYYYY
      YYyYYYYYY
      .YYyyYYY
      ..YYYYY
      ...o.o
      ..oo.oo`),
    sp('Kaninchen', `
      ..gg..gg
      ..gp..pg
      ..gp..pg
      ..gg..gg
      .gggggggg
      .gkggggkg
      .gggppggg
      ..gggggg
      .gggggggg
      gggggggggg
      gggggggggg
      .gg....gg`),
  ],
  blind_eye: [
    sp('Zwinkernder Smiley', `
      .....YYYYYY
      ...YYYYYYYYYY
      ..YYYYYYYYYYYY
      ..YYYkYYYYYYYY
      ..YYYkYYYkkkYY
      ..YYYYYYYYYYYY
      ..YYkYYYYYYkYY
      ..YYYkkkkkkYYY
      ..YYYYYYYYYYYY
      ...YYYYYYYYYY
      .....YYYYYY`),
    sp('Augenklappe', `
      ....ssssss
      ..ssssssssss
      .ssssssssssss
      kkkkkkkkkkkkkk
      .skkkssssskss
      .skkkssssssss
      .ssssssssssss
      .sssssskkksss
      ..ssssssssss
      ....ssssss`),
    sp('Halb geschlossenes Auge', `
      .....ssssss
      ...ssssssssss
      .ssssssssssssss
      kkkkkkkkkkkkkkkk
      .wwwwGGkkGGwwww
      ...wwwGGGGwww
      .....wwwwww`),
  ],
  beer_mat: [
    sp('Deckel mit Strichen', `
      .....wwwwww
      ...wwwwwwwwww
      ..wwwwwwwwwwww
      ..wwkwkwkwkwrw
      ..wwkwkwkwkrww
      ..wwkwkwkwrwww
      ..wwkwkwkrkwww
      ..wwkwkwrwkwww
      ..wwkwkrkwkwww
      ..wwwwrwwwwwww
      ...wrwwwwwwww
      .....wwwwww`),
    sp('Deckelstapel', `
      ...wwwwwwww
      .wwwrrrrrrwww
      wwwrwwwwwwrwww
      wwwrwwwwwwrwww
      .wwwrrrrrrwww
      ...wwwwwwww
      .llllllllllll
      ...llllllll
      .bbbbbbbbbbbb
      ...bbbbbbbb`),
    sp('Bleistift und Striche', `
      ...........bb
      ..........YYb
      .........YYY
      ........YYY
      .......YYY
      ......YYY
      .....sYY
      .....ks
      G.G.G.G
      G.G.GGG
      G.GGG.G
      GGG.G.G
      G.G.G.G`),
  ],
  lucky_streak: [
    sp('Kleeblatt', `
      ...eee....eee
      ..emeee..eeeme
      ..eeeeeeeeeeee
      ...eeeeeeeeee
      ....eeeMMeee
      ...eeeeeeeeee
      ..eeeeeeeeeeee
      ..eeeee..eeeee
      ...eee..b.eee
      .........b
      ..........b`),
    sp('Hufeisen', `
      ggg....ggg
      gkg....gkg
      ggg....ggg
      gkg....gkg
      ggg....ggg
      gggg..gggg
      .gggggggg
      ..gggggg`),
    sp('Marienkäfer', `
      ....kkk
      ...kwkwk
      .rrrrkrrrr
      rrkkrkrkkrr
      rrkkrkrkkrr
      rrrrrkrrrrr
      rkkrrkrrkkr
      rkkrrkrrkkr
      .rrrrkrrrr
      ..rrrkrrr`),
  ],
  tailor: [
    sp('Schere', `
      ..g..........g
      ...g........g
      ...gg......gg
      ....gg....gg
      .....gg..gg
      ......gkkg
      .......gg
      ......r..r
      .....r....r
      ...rrrr..rrrr
      ..rr..r..r..rr
      ..r...r..r...r
      ..rr.rr..rr.rr
      ...rrr....rrr`),
    sp('Garnrolle und Nadel', `
      bbbbbbbb.g
      .rrrrrr.gwg
      .rRrrrr..g
      .rrRrrr..g
      .rrrRrr..g
      .rrrrRr..g
      .rrrrrr..g
      bbbbbbbb.g
      ......rr.g
      ........rr`),
    sp('Schneiderpuppe', `
      ...lll
      .lllllll
      lllllllll
      lllllllll
      .lllllll
      ..lllll
      .lllllll
      lllllllll
      ykyykyyky
      ....b
      ....b
      ..bbbbb`),
  ],
  null_eight_fifteen: [
    sp('Schild 08/15', `
      ..wwwwwwwwwwww
      ..wwkkkwwkkkww
      ..wwkwkwwkwkww
      ..wwkwkwwkkkww
      ..wwkwkwwkwkww
      ..wwkkkwwkkkww
      ..wwwwwwwwwwww
      ..wwwkwwwkkkww
      ..wwkkwwwkwwww
      ..wwwkwwwkkkww
      ..wwwkwwwwwkww
      ..wwkkkwwkkkww
      ..wwwwwwwwwwww`),
    sp('Durchschnittsgesicht', `
      ...gggggg
      .gggggggggg
      gggggggggggg
      gggkggggkggg
      gggggggggggg
      gggggggggggg
      ggggkkkkgggg
      .gggggggggg
      ...gggggg`),
    sp('Große Null', `
      ..GGGGG
      .GGGGGGG
      GGG...GGG
      GG.....GG
      GG.....GG
      GG.....GG
      GG.....GG
      GG.....GG
      GGG...GGG
      .GGGGGGG
      ..GGGGG`),
  ],
  fat_sow: [
    sp('Dickes Schwein', `
      ..........p
      ....ppppppppp
      .p.pppppppppkp
      ..pppppppppppRR
      ..pppppppppppRR
      .ppppppppppppp
      ..pppppppppppp
      ...pppppppppp
      ...pp.pp..pp.pp
      ...pp.pp..pp.pp`),
    sp('Schweinskopf', `
      pp........pp
      ppp......ppp
      .pppppppppp
      pppppppppppp
      ppkppppppkpp
      pppppppppppp
      pppRRRRRRppp
      pppRkRRkRppp
      pppRRRRRRppp
      .pppppppppp
      ..pppppppp`),
    sp('Daus mit Rüssel', `
      .wwwwwwww
      wYwwwwwwww
      wwwwwwwwww
      wwwppppwww
      wwppppppww
      wwpkppkpww
      wwppppppww
      wwwppppwww
      wwwwwwwwww
      wwwwwwwwYw
      .wwwwwwww`),
  ],
  with_four: [
    sp('Große Vier mit Farben', `
      .JJ..........ee
      .JJ..GG..GG..ee
      .....GG..GG
      .....GG..GG
      .....GG..GG
      .....GGGGGGG
      .....GGGGGGG
      .........GG
      .........GG
      .........GG
      .rr......GG..LL
      .rr..........LL`),
    sp('Vier Unter', `
      JJJ.eee.rrr.LLL
      JJJ.eee.rrr.LLL
      sss.sss.sss.sss
      sks.sks.sks.sks
      sss.sss.sss.sss
      bbb.eee.rrr.yyy
      bbb.eee.rrr.yyy
      bbb.eee.rrr.yyy`),
    sp('Vier Finger', `
      ss.ss.ss.ss
      ss.ss.ss.ss
      ss.ss.ss.ss
      ss.ss.ss.ss
      sssssssssss
      sssssssssss
      ssssssssss
      .ssssssss
      ..ssssss
      ..rrrrrr
      ..rrrrrr`),
  ],
  // ---- M8 wave 3, Legendär (one motif each, drawn for the wave, not voted)
  skat_god: [
    sp('Gekrönter Kopf mit Heiligenschein', `
      ....YYYYYYY
      ...Y.......Y
      ....YYYYYYY
      ...y..y.y..y
      ...yyyyyyyyy
      ...yryyyyyry
      ...yyyyyyyyy
      ....sssssss
      ....skssska
      ....sssssss
      ....wskkksw
      ...wwwsssww
      ..wwwwwwwwww
      ..wwwwwwwwww`),
  ],
  honorary_chair: [
    sp('Lehnstuhl mit goldenem Stern', `
      ..YnnnnnnnnnY
      ..nnnnnnnnnnn
      ..nnrrrrrrrnn
      ..nnrrrYrrrnn
      ..nnrrYYYrrnn
      ..nnrrrYrrrnn
      ..nnrrrrrrrnn
      ..nnnnnnnnnnn
      .nrrrrrrrrrrrn
      .nnnnnnnnnnnnn
      .nn.........nn
      .nn.........nn
      .nn.........nn`),
  ],
  // ---- M8 wave 2a (one motif each, drawn for the wave, not voted)
  skat_keeper: [
    sp('Zwei Karten mit rotem Kreuz', `
      wwwwwww
      wwwrwww
      wwrrrww
      wwwrwww.wwwwwww
      wwwwwww.wwwrwww
      wwwwwww.wwrrrww
      wwwwwww.wwwrwww
      wwwwwww.wwwwwww
      ........wwwwwww
      ........wwwwwww`),
  ],
  all_rounder: [
    sp('Taschenmesser', `
      gggggggg
      .gggggggggg
      ...rrrrrrrrrr
      ..rrrrrwrrrrr
      ..rrrrwwwrrrr
      ..rrrrrwrrrrr
      ...rrrrrrrrrr
      ..........g.g
      ...........gg`),
  ],
  home_suit: [
    sp('Haus mit rotem Herz', `
      ......OO
      .....OOOO
      ....OOOOOO
      ...OOOOOOOO
      ..OOOOOOOOOO
      .OOOOOOOOOOOO
      ..wwwwwwwwww
      ..wrrwrrwwww
      ..wrrrrrwnnw
      ..wwrrrwwnnw
      ..wwwrwwwnnw
      ..wwwwwwwnnw`),
  ],
  blank_king: [
    sp('Sieben mit Krone', `
      .y..y..y.
      .yyyyyyy.
      .yryyyry.
      .yyyyyyy.
      .........
      rrrrrrrrr
      .......rr
      ......rr
      .....rr
      ....rr
      ....rr
      ....rr`),
  ],
  watertight: [
    sp('Schild', `
      GGGGGGGGGG
      GmmmmmmmmG
      GmmmGGmmmG
      GmmmGGmmmG
      GmGGGGGGmG
      GmGGGGGGmG
      GmmmGGmmmG
      .GmmGGmmG
      .GmmmmmmG
      ..GmmmmG
      ...GmmG
      ....GG`),
  ],
  // ---- M8 wave 2b (one motif each, drawn for the wave, not voted)
  three_discard: [
    sp('Drei Karten über einem Pfeil nach unten', `
      wwww.wwww.wwww
      weww.wrww.wyww
      weew.wrrw.wyyw
      weww.wrww.wyww
      wwww.wwww.wwww
      wwww.wwww.wwww
      ..............
      ....GGGGGG
      .....GGGG
      ......GG`),
  ],
  crib_sheet: [
    sp('Zettel mit Bleistift', `
      wwwwwwwww
      wGGwGGGww
      wwwwwwwww
      wGGGwGGww....yy
      wwwwwwwww...yyy
      wGGwGGGww..yyy
      wwwwwwwww.yyy
      wGGGwGww.yyy
      wwwwwwwwlyy
      ........nl`),
  ],
  fresh_deck: [
    sp('Kartenschachtel', `
      ............h
      ...rrrrrrrrrr
      ...ryyyyyyyyr
      ...ryrrrrrryr
      ...yyyyyyyyyy
      ...ryrrwwrryr
      ...ryrwwwwryr
      ...ryrwwwwryr
      ...ryrrwwrryr
      ...ryrrrrrryr
      ...ryyyyyyyyr
      ...rrrrrrrrrr`),
    sp('Neue Karte mit Glanz', `
      .........Y
      wwwwwww.YYY
      wrwwwww..Y
      wwwwwww
      wwrwrww
      wwrrrww
      wwwrwww
      wwwwwww
      wwwwwrw
      wwwwwww`),
    sp('Kartenstapel', `
      rrrrrrrrrr
      rYrrrrrrYr
      rrrYrrYrrr
      rrrrYYrrrr
      rrrYrrYrrr
      rYrrrrrrYr
      rrrrrrrrrr
      wwwwwwwwww
      gggggggggg
      wwwwwwwwww
      gggggggggg`),
  ],
  ink_pad: [
    sp('Stempel und Kissen', `
      ......bbbb
      ......bbbb
      .......bb
      .......bb
      .....gggggg
      .....vvvvvv
      ...............
      ..GGGGGGGGGGGG
      ..GvvvvvvvvvvG
      ..GvvvvvvvvvvG
      ..GGGGGGGGGGGG`),
    sp('Stempel mit Stern', `
      ...bbbb
      ...bbbb
      ....bb
      ....bb
      ..gggggg
      ..vvvvvv
      ..........
      ....vv
      .vvvvvvvv
      ..vvvvvv
      ..vv..vv`),
    sp('Gestempelte Karte', `
      wwwwwwwww
      wrwwwwwww
      wwwvvvwww
      wwvwwwvww
      wwvwvwvww
      wwvwwwvww
      wwwvvvwww
      wwwwwwwww
      wwwwwwwrw
      wwwwwwwww`),
  ],
  shot: [
    sp('Schnapsglas', `
      ...cccccccccc
      ...chfffffffc
      ...chfffffffc
      ....chfffffc
      ....chfffffc
      ....chfffffc
      ....cffffffc
      ....cccccccc
      ....cccccccc`),
    sp('Flachmann', `
      ...kk
      ...gg
      .gggggg
      ghgggggg
      GGGGGGGG
      GGGGGGGG
      ghgggggg
      gggggggg
      .gggggg`),
    sp('Glas über dem Skat', `
      ...cccccc
      ...chfffc
      ....cffc
      ....cccc
      ..........
      rrrrr.rrrrr
      rYrYr.rYrYr
      rrrrr.rrrrr
      rYrYr.rYrYr
      rrrrr.rrrrr`),
  ],
  double_grain: [
    sp('Klare Flasche', `
      .......rr
      .......rr
      ......cccc
      ......cchc
      .....cccchc
      ....cccccchc
      ....cwwwwwwc
      ....cwrwwrwc
      ....cwwwwwwc
      ....cccccchc
      ....cccccchc
      ....cccccchc
      ....cccccccc
      .....cccccc`),
    sp('Zwei Flaschen', `
      .rr....rr
      .cc....cc
      .cc....cc
      cccc..cccc
      chcc..chcc
      wwww..wwww
      wrrw..wrrw
      wwww..wwww
      chcc..chcc
      cccc..cccc`),
    sp('Glas mit Zwei', `
      cccccccccc
      chfrrrfffc
      chffffrffc
      chffrrfffc
      chfrfffffc
      chfrrrrffc
      chfffffffc
      .cccccccc`),
  ],
  herbal: [
    sp('Grüne Flasche mit Blatt', `
      .......nn
      .......nn
      ......eeee
      ......emee
      .....emeeee
      ....emeeeeee
      ....wwwwwwww
      ....wwweewww
      ....wweemwww
      ....wwwwewww
      ....wwwwwwww
      ....emeeeeee
      ....emeeeeee
      .....eeeeee`),
    sp('Kräuterbund', `
      ....e
      ..e.eme.e
      .eme.e.eme
      .ee..e..ee
      ..e..e..e
      ...e.e.e
      ....eee
      ....rrr
      ....eee
      .....e
      ....e.e`),
    sp('Hirschkopf', `
      b..b....b..b
      bbbb....bbbb
      ..bb....bb
      ...bb..bb
      ...llllll
      ..llllllll
      ..lkllllkl
      ...llllll
      ....llll
      ....lnnl
      .....ll`),
  ],
  blue_curacao: [
    sp('Blaue Likörflasche', `
      .......kk
      .......uu
      .......uu
      ......uuuu
      .....uuuuuu
      ....uuuuuuuu
      ....uhuuuuuu
      ....wwwwwwww
      ....wwwoowww
      ....wwolloww
      ....wwwoowww
      ....wwwwwwww
      ....uhuuuuuu
      ....uuuuuuuu
      ....UUUUUUUU`, CURACAO),
    sp('Cocktailglas mit Orangenscheibe', `
      ...........ooo
      ..........olllo
      cccccccccccolo
      .cuhuuuuuuuuuc
      ..cuhuuuuuuuc
      ...cuuuuuuuc
      ....cuuuuuc
      .....cuuuc
      ......ccc
      .......c
      .......c
      .......c
      .....ccccc`, CURACAO),
  ],
  // ---- motifs of the Kneipentour relics and Schnäpse (emblems v2, picked
  // by the user in the art vote "Deckel v2"; counter_beer and snifter
  // still have two variants for the vote)
  closing_time: [
    sp('Garderobe mit Hut', `
      .....nnnnn
      .....nnnnn
      .....rrrrr
      ...nnnnnnnnn
      ......gbg
      .....g.b.g
      .......b.JJ
      .......b.JJJ
      .......b.JJJ
      .......b..J
      .......b
      .......b
      .....bbbbb
      ....bb...bb`),
  ],
  season_ticket: [
    sp('Fahrkarte mit Foto', `
      JJJJJJJJJJJJJJ
      JJwJwJwJJJJLLJ
      JJJJJJJJJJJJJJ
      wwwwwwwwwwwwww
      wnnnnwwwwwwwww
      wsssswwggggggw
      wskskwwwwwwwww
      wsssswwggggwww
      wJJJJwwwwwwwww
      wJJJJwwrrwrrww
      wwwwwwwwwwwwww`),
  ],
  kitty: [
    sp('Sparschwein', `
      ........YYY
      ........YyY
      ..p.....kkk
      .ppppppppppp
      .pkpppppppppp
      RRppppppppppppp
      RRpppppppppppp.p
      .pppppppppppp
      ..ppppppppppp
      ..pp.....pp
      ..RR.....RR`),
  ],
  slate: [
    sp('Schiefertafel mit Strichen', `
      bbbbbbbbbbbbbb
      bddddddddddddb
      bdwdwdwdwddddb
      bdwdwdwdwddddb
      bdwwwwwwwwdddb
      bdwdwdwdwddddb
      bddddddddddddb
      bdwdwdwddaaddb
      bdwdwdwddahddb
      bddddddddaaddb
      bbbbbbbbbbbbbb`),
  ],
  guardian_angel: [
    sp('Herz mit Flügeln', `
      ....YYYYYYY
      ...Y.......Y
      ....YYYYYYY
      ...............
      ww..rrr.rrr..ww
      www.rRrrrrr.www
      wwwwrrrrrrrwwww
      .wwwrrrrrrrwww
      ..wwwrrrrrwww
      ...ww.rrr.ww
      .......r`),
  ],
  dive_expert: [
    sp('Lupe vor schiefer Tür', `
      ...bbbbb
      ...bnnnb
      ..bbnnnb
      ..bnnnnb.GGGG
      ..bnnnnbGccccG
      ..bnnnLGchccccG
      .bnnnnbGchccccG
      .bnnnnbGccccccG
      .bnnnnbGccccccG
      .bnnnnb.GccccG
      bnnnnnb..GGGGgg
      bnnnnnb......ggg
      bnnnnnb.......gg
      bbbbbbb`),
  ],
  counter_beer: [
    sp('Krug mit Eisbeutel', `
      .....gg
      ....cccc
      ...cchccc
      ..ccccccccc
      ...wwwwwwwww
      ..wwwwwwwwwww
      ..aaaaaaaaaaggg
      ..ahaaaaaaaa..g
      ..ahaaaaaaaa..g
      ..ahaaaaaaaa..g
      ..aaaaaaaaaaggg
      ..aaaaaaaaaa
      ..gggggggggg`),
    sp('Schäumender Krug mit Herz', `
      ...w..ww
      ..wwwwwwww
      .wwwwwwwwwww
      .wwwwwwwwwwww
      .waaaaaaaaaaggg
      ..aharrarraa..g
      ..aharrrrraa..g
      ..ahaarrraaa..g
      ..ahaaaraaaa..g
      ..aaaaaaaaaaggg
      ..aaaaaaaaaa
      ..gggggggggg`),
  ],
  snifter: [
    sp('Schwenker mit Wirbel', `
      ....ccccc
      ...chccccc
      ..chccccccc
      .chooooooooc
      choYYYooooooc
      cooooooooYooc
      .coYoooooYoc
      ..coYYYYYoc
      ...ccccccc
      ......c
      ......c
      ....ccccc`),
    sp('Schwenker mit Wendepfeil', `
      ....rrrrr
      ..rr.....rr
      .r.........r
      rrr........r
      .r..ccccc
      ...chccccc
      ..chooooooc
      ..cooYYYooc
      ..coooooooc
      ...ccccccc
      ......c
      ....ccccc`),
  ],
  road_beer: [
    sp('Flasche in der Hand', `
      .....YY
      .....ll
      .....bb
      ....bbbb
      ...bbbbbb
      ...blbbbb
      ...bwwwwb
      ...bwrrwb
      ..sssssssss
      ..kkkkkksss
      ..sssssssss
      ..kkkkkksss
      ..ssssssssss
      ...bbbbb.ssss
      ...bbbbb..sss`),
  ],
  // ---- rescue Schnäpse (DESIGN 8.7, 03.11.2026), one motif each
  clear_spirit: [
    sp('Klare Flasche mit Kappe', `
      ......GGG
      ......ggg
      ......chc
      ......chc
      .....chhcc
      ....cchcccc
      ...cchcccccc
      ...chccccccc
      ...chwwwwwwc
      ...chwJJJJwc
      ...chwwwwwwc
      ...chccccccc
      ...chccccccc
      ...cccccccccc`),
  ],
  nightcap: [
    sp('Glas unter dem Mond', `
      ..........LLL
      .........LL
      .........L
      .........LL
      ..........LLL
      ..............
      .cccccccccc
      .chcccccccc
      .choooooooc
      .choooooooc
      .cOOOOOOOOc
      .cccccccccc`),
  ],
  eggnog: [
    sp('Eierlikör im Gläschen', `
      .cccccccccc
      .chYLLLLLLc
      .chLLLLLLLc
      ..chLLLLLc....ww
      ...cLLLLc....wwww
      ....cccc.....wwww
      .....cc......wwww
      .....cc.......ww
      ....cccc
      ...cccccc`),
  ],
  // Glasauge (04.11.2026): an eye behind a gold monocle with its chain,
  // on the rounded square Control mat.
  glass_eye: [
    sp('Auge mit Monokel', `
      ....yyyyyyy
      ..yyhhhhhhhyy
      .yhccccccccchy
      .ycwwwwwwwwwcy
      ycwwwJJJJwwwwcy
      ycwwJJkkJJwwwcy
      ycwwJJkkhJwwwcy
      ycwwwJJJJwwwwcy
      .ycwwwwwwwwwcy
      .yhccccccccchy
      ..yyhhhhhhhyy.y
      ....yyyyyyy...y
      ..............y
      .............y`),
  ],
  // ---- curses as trade (03.11.2026), one motif each
  // Pechvogel: a black cat with yellow eyes, on the round Value mat.
  jinx: [
    sp('Schwarze Katze', `
      ..d.......d
      ..dd.....dd
      ..ddddddddd
      .ddddddddddd
      .ddLLdddLLdd
      .ddLkdddkLdd
      .dddddpddddd
      ..ddddddddd
      ...ddddddd
      ..ddddddddd....d
      .dddGddddddd...d
      .dddGddddddd..dd
      .ddddddddddddddd
      .dddddddddddddd
      ..dd.ddd.dd`),
  ],
  // Fusel: a dirty bottle without a label, murky glass, muddy spirit.
  rotgut: [
    sp(
      'Schmutzige Flasche ohne Etikett',
      `
      .......nn
      .......nn
      ......gyyg
      ......gccg
      .....gcchcg
      ....gccchccg
      ...gcOcchcccg
      ...gccchccccg
      ...gccchcOccg
      ...gyfffffffg
      ...gffOffffyg
      ...gfffffOffg
      ...gfffffffyg
      ....ggggggggg`,
      { c: '#7d8a5e', f: '#b59b52' },
    ),
  ],
};

/** One motif per boss (core/content/bosses.ron). */
export const BOSS_ART: Record<string, Sprite> = {
  the_pusher: sp('Zylinder und Schnurrbart', `
    .....kkkkkk
    .....kkkkkk
    .....kkkkkk
    .....kkkkkk
    .....rrrrrr
    ...kkkkkkkkkk
    ...............
    ....ssssssss
    ....skssssks
    ....ssssssss
    ....snnnnnns
    ....ssssssssoo
    .....ssssss..g
    ......ssss...g`),
  grandma: sp('Handtasche voller Unter', `
    ......bbbb
    .....b....b
    ....b.wwww.b
    ....b.wyyw.b
    ...pppwrrwppp
    ...pppppppppp
    ...ppppyyyppp
    ...pppppppppp
    ...pppppppppp
    ...pppppppppp
    ....pppppppp`),
  twins: sp('Zwei gleiche Gesichter', `
    .nnnnnn..nnnnnn
    .nssssn..nssssn
    .skssks..skssks
    .ssssss..ssssss
    .sskkss..sskkss
    ..ssss....ssss
    .oooooo..oooooo
    .owowow..owowow
    .oooooo..oooooo`),
  ramscher: sp('Mülleimer mit Karten', `
    ....ww....rr
    ....ww...rr
    ..gggggggggggg
    ...gGggGggGgg
    ...gGggGggGgg
    ...gGggGggGgg
    ...gGggGggGgg
    ...gGggGggGgg
    ...gGggGggGgg
    ....gggggggg`),
  black_peter: sp('Karte mit schwarzer Katze', `
    ....wwwwwwww
    ...wwwwwwwwww
    ...wwkwwwwkww
    ...wwkkwwkkww
    ...wwkkkkkkww
    ...wkkkkkkkkw
    ...wkYkkkkYkw
    ...wkkkkkkkkw
    ...wkkkppkkkw
    ...wwkkkkkkww
    ...wwwkkkkwww
    ...wwwwwwwwww
    ....wwwwwwww`),
  the_old_man: sp('Alter König mit Bart', `
    ....y..yy..y
    ....yyyyyyyy
    ....yryyyyry
    ...gssssssssg
    ...gskssssksg
    ...gssssssssg
    ...wwwwwwwwww
    ...wwwwwwwwww
    ....wwwwwwww
    ....wwwwwwww
    .....wwwwww
    ......wwww
    .......ww`),
};

/** A generic motif per kind, for ids without their own. */
const FALLBACK: Record<RelicKind | 'Schnapps' | 'Item' | 'Boss', Sprite> = {
  HouseRule: sp('Paragraf', `
    ......GGGG
    .....GG..GG
    .....GG
    ......GGGG
    .....GG..GG
    .....GG..GG
    ......GGGG
    .........GG
    .....GG..GG
    ......GGGG`),
  Regular: sp('Figur', `
    ......nnnn
    .....nnnnnn
    .....ssssss
    .....ssssss
    ......ssss
    .......ss
    ....GGGGGGGG
    ...GGGGGGGGGG
    ..GGGGGGGGGGGG
    ..GGGGGGGGGGGG`),
  Schnapps: sp('Flasche', `
    .......rr
    .......cc
    .......cc
    ......cccc
    .....cccccc
    .....chcccc
    .....chcccc
    .....chcccc
    .....chcccc
    .....cccccc`),
  Item: sp('Paket', `
    .....nn..nn
    ...bbbbnnbbbb
    ...bbbbnnbbbb
    ...bbbbnnbbbb
    ...nnnnnnnnnn
    ...bbbbnnbbbb
    ...bbbbnnbbbb
    ...bbbbnnbbbb`),
  Boss: sp('Schatten mit roten Augen', `
    .....GGGGGG
    ....GGGGGGGG
    ...GGGGGGGGGG
    ...GGkkkkkkGG
    ...GGkrkkrkGG
    ...GGkkkkkkGG
    ....GGkkkkGG
    ...GGGGGGGGGG
    ..GGGGGGGGGGGG
    ..GGGGGGGGGGGG`),
};

/**
 * The motif shown for each relic and consumable: 0, 1 or 2 (variant A, B
 * or C of SHOP_ART). Missing ids show variant A.
 */
// Chosen by the user in the cockpit on 01.11.2026 (Tresen-Grafiken).
export const CHOSEN: Record<string, 0 | 1 | 2> = {
  last_trick_seven: 1,
  concede: 0,
  skat_redraw: 2,
  virgin: 0,
  old_lady: 0,
  tour: 1,
  kibitzer: 0,
  landlord: 0,
  card_counter: 2,
  every_bit_counts: 0,
  blind_eye: 1,
  beer_mat: 0,
  lucky_streak: 0,
  tailor: 0,
  null_eight_fifteen: 0,
  fat_sow: 1,
  with_four: 1,
  fresh_deck: 0,
  ink_pad: 0,
  shot: 0,
  double_grain: 1,
  herbal: 0,
  // Deckel v2 (02.11.2026): one variant kept each.
  closing_time: 0,
  season_ticket: 0,
  kitty: 0,
  slate: 0,
  guardian_angel: 0,
  dive_expert: 0,
  road_beer: 0,
  clear_spirit: 0,
  nightcap: 0,
  eggnog: 0,
  // Curses as trade (03.11.2026): one motif each.
  jinx: 0,
  // Glasauge (04.11.2026): one motif.
  glass_eye: 0,
  rotgut: 0,
  // Still to be voted on; variant A until then.
  blue_curacao: 0,
  counter_beer: 0,
  snifter: 0,
};

// ---------------------------------------------------------------------------
// Sprites to SVG
// ---------------------------------------------------------------------------

const GRID = 16;

/** The rows of a sprite, trimmed, every row padded to the same width. */
export function spriteRows(s: Sprite): string[] {
  const rows = s.art
    .split('\n')
    .map((r) => r.trim())
    .filter((r, i, all) => r.length > 0 || (i > 0 && i < all.length - 1));
  while (rows.length && rows[0] === '') rows.shift();
  const width = Math.max(0, ...rows.map((r) => r.length));
  return rows.map((r) => r.padEnd(width, '.'));
}

const spriteCache = new Map<Sprite, string>();

/**
 * Paths of a sprite in grid units (16 × 16), centred by its bounding box,
 * with a one-pixel dark outline around the motif.
 */
export function spriteMarkup(s: Sprite): string {
  const hit = spriteCache.get(s);
  if (hit !== undefined) return hit;
  const rows = spriteRows(s);
  const cells: { x: number; y: number; ch: string }[] = [];
  rows.forEach((row, y) => [...row].forEach((ch, x) => ch !== '.' && ch !== ' ' && cells.push({ x, y, ch })));
  if (!cells.length) return '';
  const minX = Math.min(...cells.map((c) => c.x));
  const maxX = Math.max(...cells.map((c) => c.x));
  const minY = Math.min(...cells.map((c) => c.y));
  const maxY = Math.max(...cells.map((c) => c.y));
  const ox = Math.floor((GRID - (maxX - minX + 1)) / 2) - minX;
  const oy = Math.floor((GRID - (maxY - minY + 1)) / 2) - minY;
  const filled = new Set(cells.map((c) => `${c.x + ox},${c.y + oy}`));
  const byColour = new Map<string, Set<string>>();
  const put = (colour: string, x: number, y: number): void => {
    let set = byColour.get(colour);
    if (!set) byColour.set(colour, (set = new Set()));
    set.add(`${x},${y}`);
  };
  // The outline: every empty cell next to the motif (four neighbours).
  for (const key of filled) {
    const [x, y] = key.split(',').map(Number) as [number, number];
    for (const [dx, dy] of [
      [1, 0],
      [-1, 0],
      [0, 1],
      [0, -1],
    ] as const) {
      const n = `${x + dx},${y + dy}`;
      if (!filled.has(n)) put(OUTLINE, x + dx, y + dy);
    }
  }
  const pal = { ...PALETTE, ...(s.pal ?? {}) };
  for (const c of cells) put(pal[c.ch] ?? OUTLINE, c.x + ox, c.y + oy);
  let out = '';
  for (const [colour, set] of byColour) {
    // Horizontal runs per row, one path per colour.
    const pts = [...set].map((k) => k.split(',').map(Number) as [number, number]).sort((a, b) => a[1] - b[1] || a[0] - b[0]);
    let d = '';
    let i = 0;
    while (i < pts.length) {
      const [x0, y0] = pts[i] as [number, number];
      let len = 1;
      while (i + len < pts.length && pts[i + len]?.[1] === y0 && pts[i + len]?.[0] === x0 + len) len++;
      d += `M${x0} ${y0}h${len}v1h-${len}z`;
      i += len;
    }
    out += `<path fill="${colour}" d="${d}"/>`;
  }
  spriteCache.set(s, out);
  return out;
}

// ---------------------------------------------------------------------------
// Forms (viewBox 0 0 100 100, centre 50 48)
// ---------------------------------------------------------------------------

type Pt = [number, number];

const CX = 50;
const CY = 48;

function poly(points: Pt[]): string {
  return points.length ? 'M' + points.map(([x, y]) => `${x.toFixed(1)} ${y.toFixed(1)}`).join('L') + 'Z' : '';
}

function radialPts(n: number, radius: (k: number) => number, phase = 0): Pt[] {
  const pts: Pt[] = [];
  for (let k = 0; k < n; k++) {
    const a = phase + (k * 2 * Math.PI) / n;
    const r = radius(k);
    pts.push([CX + r * Math.cos(a), CY + r * Math.sin(a)]);
  }
  return pts;
}

/** Points of a circular arc from angle a0 to a1 (radians), both ends included. */
function arcPts(cx: number, cy: number, r: number, a0: number, a1: number, n: number): Pt[] {
  return Array.from({ length: n + 1 }, (_, k): Pt => {
    const a = a0 + ((a1 - a0) * k) / n;
    return [cx + r * Math.cos(a), cy + r * Math.sin(a)];
  });
}

/** Points of a quadratic Bézier from p0 over c to p1, without p0. */
function quadPts(p0: Pt, c: Pt, p1: Pt, n: number): Pt[] {
  return Array.from({ length: n }, (_, k): Pt => {
    const t = (k + 1) / n;
    const u = 1 - t;
    return [u * u * p0[0] + 2 * u * t * c[0] + t * t * p1[0], u * u * p0[1] + 2 * u * t * c[1] + t * t * p1[1]];
  });
}

function signedArea(pts: Pt[]): number {
  let s = 0;
  pts.forEach(([x0, y0], i) => {
    const [x1, y1] = pts[(i + 1) % pts.length] as Pt;
    s += x0 * y1 - x1 * y0;
  });
  return s / 2;
}

/**
 * The outline moved inward by d (mitred corners). Good for the small
 * insets of rim, band and ring; the forms have no features thinner than
 * twice the largest inset.
 */
function insetPts(pts: Pt[], d: number): Pt[] {
  if (d === 0) return pts;
  const s = Math.sign(signedArea(pts)) || 1;
  const n = pts.length;
  const normal = (a: Pt, b: Pt): Pt => {
    const dx = b[0] - a[0];
    const dy = b[1] - a[1];
    const len = Math.hypot(dx, dy) || 1;
    return [(-dy / len) * s, (dx / len) * s];
  };
  return pts.map((p, i): Pt => {
    const prev = pts[(i - 1 + n) % n] as Pt;
    const next = pts[(i + 1) % n] as Pt;
    const n1 = normal(prev, p);
    const n2 = normal(p, next);
    const k = Math.max(0.3, 1 + n1[0] * n2[0] + n1[1] * n2[1]);
    return [p[0] + (d * (n1[0] + n2[0])) / k, p[1] + (d * (n1[1] + n2[1])) / k];
  });
}

/** Clip a polygon to one side of a line x = v (axis 0) or y = v (axis 1). */
function clipPts(pts: Pt[], axis: 0 | 1, v: number, keepAbove: boolean): Pt[] {
  const inside = (p: Pt): boolean => (keepAbove ? p[axis] >= v : p[axis] <= v);
  const out: Pt[] = [];
  pts.forEach((p, i) => {
    const q = pts[(i + 1) % pts.length] as Pt;
    if (inside(p)) out.push(p);
    if (inside(p) !== inside(q)) {
      const t = (v - p[axis]) / (q[axis] - p[axis]);
      out.push([p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t]);
    }
  });
  return out;
}

/** Clip a polygon to the box x0..x1, y0..y1. */
function clipBox(pts: Pt[], x0: number, y0: number, x1: number, y1: number): Pt[] {
  return clipPts(clipPts(clipPts(clipPts(pts, 0, x0, true), 0, x1, false), 1, y0, true), 1, y1, false);
}

function roundedSquarePts(): Pt[] {
  const [x0, y0, x1, y1, r] = [6, 4, 94, 92, 16];
  const q = Math.PI / 2;
  return [
    ...arcPts(x1 - r, y0 + r, r, -q, 0, 6),
    ...arcPts(x1 - r, y1 - r, r, 0, q, 6),
    ...arcPts(x0 + r, y1 - r, r, q, 2 * q, 6),
    ...arcPts(x0 + r, y0 + r, r, 2 * q, 3 * q, 6),
  ];
}

function shieldPts(): Pt[] {
  return [
    [14, 6],
    [86, 6],
    [90, 10],
    [90, 50],
    ...quadPts([90, 50], [90, 80], [50, 94], 8),
    ...quadPts([50, 94], [10, 80], [10, 50], 8),
    [10, 10],
  ];
}

function bottlePts(): Pt[] {
  return [
    [40, 3],
    [60, 3],
    [60, 22],
    ...quadPts([60, 22], [86, 24], [86, 44], 6),
    [86, 89],
    ...quadPts([86, 89], [86, 95], [80, 95], 3),
    [20, 95],
    ...quadPts([20, 95], [14, 95], [14, 89], 3),
    [14, 44],
    ...quadPts([14, 44], [14, 24], [40, 22], 6),
  ];
}

/**
 * A Schnaps label: portrait, with concave cut corners and a cork tab on
 * top, so it reads as a bottle label and not as a mat even at 40 px.
 */
function labelPts(): Pt[] {
  const [x0, y0, x1, y1, r] = [18, 24, 82, 97, 10];
  const [n0, n1, top] = [36, 64, 2];
  const q = Math.PI / 2;
  return [
    [n0, top],
    [n1, top],
    [n1, y0],
    ...arcPts(x1, y0, r, 2 * q, q, 5),
    ...arcPts(x1, y1, r, 3 * q, 2 * q, 5),
    ...arcPts(x0, y1, r, 4 * q, 3 * q, 5),
    ...arcPts(x0, y0, r, q, 0, 5),
    [n0, y0],
  ];
}

function shotGlassPts(): Pt[] {
  return [
    [8, 8],
    [92, 8],
    [81, 84],
    [83, 94],
    [17, 94],
    [19, 84],
  ];
}

/** Fixed jitter of the torn boss edge (a small seeded generator, looks only). */
const TORN_JITTER: number[] = (() => {
  let seed = 7;
  return Array.from({ length: 48 }, () => {
    seed = (seed * 1103515245 + 12345) % 2147483648;
    return -3 + (seed / 2147483648) * 5;
  });
})();

/** How a Schnaps is drawn: S1 bottle, S2 label, S3 shot glass (art vote "Deckel v2"). */
export type SchnapsStyle = 'Bottle' | 'Label' | 'Glass';

/** How the rarity is marked: R1 rim band, R2 corner gem, R3 ring and pips. */
export type RarityStyle = 'Band' | 'Gem' | 'Pips';

/** The style in use; the board of the art vote overrides it per call. */
export const EMBLEM_STYLE: { rarity: RarityStyle; schnaps: SchnapsStyle } = {
  rarity: 'Band',
  schnaps: 'Label',
};

/**
 * Category per relic id, only a fallback: the core sends `group` with
 * every relic (RelicInfo, catalog); an older core or save without it gets
 * the shape from here.
 */
export const RELIC_GROUP: Record<string, RelicGroup> = {
  // Grundwert and Kartenwerte
  all_rounder: 'Value',
  beer_mat: 'Value',
  tour: 'Value',
  null_eight_fifteen: 'Value',
  every_bit_counts: 'Value',
  fat_sow: 'Value',
  old_lady: 'Value',
  skat_keeper: 'Value',
  jinx: 'Value',
  // Multiplikator, Gewinngrenze and Schneider
  blank_king: 'Multiplier',
  last_trick_seven: 'Multiplier',
  with_four: 'Multiplier',
  skat_god: 'Multiplier',
  home_suit: 'Multiplier',
  lucky_streak: 'Multiplier',
  blind_eye: 'Multiplier',
  tailor: 'Multiplier',
  virgin: 'Multiplier',
  watertight: 'Multiplier',
  concede: 'Multiplier',
  // Geben, Skat and Durchblick
  skat_redraw: 'Control',
  three_discard: 'Control',
  crib_sheet: 'Control',
  honorary_chair: 'Control',
  card_counter: 'Control',
  kibitzer: 'Control',
  glass_eye: 'Control',
  // Tresen and Tour
  landlord: 'Tour',
  closing_time: 'Tour',
  season_ticket: 'Tour',
  kitty: 'Tour',
  slate: 'Tour',
  guardian_angel: 'Tour',
  dive_expert: 'Tour',
};

/** German names of the categories, for the art vote board only. */
export const GROUP_LABEL: Record<RelicGroup, string> = {
  Value: 'Grundwert und Kartenwerte',
  Multiplier: 'Multiplikator und Gewinngrenze',
  Control: 'Geben, Skat und Durchblick',
  Tour: 'Tresen und Tour',
};

/** Every drawn form: four relic mats, the item mat, the boss mat and three Schnaps styles. */
export type Form = RelicGroup | 'Item' | 'Boss' | SchnapsStyle;

interface FormSpec {
  /** Outer silhouette. */
  pts: Pt[];
  paper: string;
  /** Motif box: left, top, scale of the 16-pixel grid. */
  motif: [number, number, number];
  /** Where the rarity pips sit on the rim. */
  pips: 'top' | 'bottom';
  /** Fixed spot of the rarity gem; default: the corner top right. */
  gem?: Pt;
  /** Bottom of a cork drawn inside the paper above this y. */
  cork?: number;
  /** Glass: level of the spirit, start of a thick base, highlight stripe x. */
  glass?: { liquid: number; base?: number; shine: [number, number, number, number] };
}

const GLASS = '#d3eaf0';
const SPIRIT = '#f4e9bf';

const FORMS: Record<Exclude<Form, 'Boss'>, FormSpec> = {
  // Grundwert: the plain round beer mat.
  Value: { pts: radialPts(72, () => 44), paper: '#ecebe4', motif: [22, 20, 3.5], pips: 'bottom' },
  // Multiplikator: a sunburst with twelve points.
  Multiplier: {
    pts: radialPts(24, (k) => (k % 2 ? 39.5 : 47), -Math.PI / 2),
    paper: '#f1dfb8',
    motif: [22, 20, 3.5],
    pips: 'bottom',
    gem: [80, 18],
  },
  // Kontrolle: a rounded square like a card table.
  Control: { pts: roundedSquarePts(), paper: '#d6e2ec', motif: [22, 20, 3.5], pips: 'bottom' },
  // Tour: an inn sign shield.
  Tour: { pts: shieldPts(), paper: '#d8ebdf', motif: [22, 16, 3.5], pips: 'top' },
  // Other consumables (not Schnaps): the octagonal mat.
  Item: { pts: radialPts(8, () => 47, Math.PI / 8), paper: '#e4d2bd', motif: [22, 20, 3.5], pips: 'bottom' },
  // S1: a bottle, the motif on its belly.
  Bottle: {
    pts: bottlePts(),
    paper: GLASS,
    motif: [24, 39, 3.25],
    pips: 'bottom',
    gem: [77, 36],
    glass: { liquid: 48, shine: [19, 44, 23, 86] },
  },
  // S2: a portrait paper label with cut corners and a cork on top.
  Label: { pts: labelPts(), paper: '#f7f0dc', motif: [26, 36, 3], pips: 'bottom', gem: [74, 33], cork: 22 },
  // S3: a shot glass with a thick base.
  Glass: {
    pts: shotGlassPts(),
    paper: GLASS,
    motif: [23, 15, 3.4],
    pips: 'bottom',
    gem: [84, 14],
    glass: { liquid: 22, base: 84, shine: [16, 14, 20, 72] },
  },
};

const pathCache = new Map<string, string>();
const ptsCache = new Map<string, Pt[]>();

function formPts(form: Exclude<Form, 'Boss'>, inset: number): Pt[] {
  const key = `${form}:${inset}`;
  let p = ptsCache.get(key);
  if (p === undefined) ptsCache.set(key, (p = insetPts(FORMS[form].pts, inset)));
  return p;
}

function formPath(form: Form, inset: number): string {
  const key = `${form}:${inset}`;
  let d = pathCache.get(key);
  if (d === undefined) {
    d =
      form === 'Boss'
        ? poly(radialPts(48, (k) => 43 - inset + (inset === 0 ? (TORN_JITTER[k] ?? 0) : 0)))
        : poly(formPts(form, inset));
    pathCache.set(key, d);
  }
  return d;
}

/**
 * The point of an outline farthest in a direction; among nearly equal
 * ones (a flat edge) the one closest to the axis through the centre.
 */
function extreme(pts: Pt[], dx: number, dy: number): Pt {
  const score = ([x, y]: Pt): number => Math.round((x * dx + y * dy) * 2) * 1000 - Math.abs((x - CX) * dy - (y - CY) * dx);
  let best = pts[0] ?? [CX, CY];
  for (const p of pts) if (score(p) > score(best)) best = p;
  return best;
}

// ---------------------------------------------------------------------------
// Rarity
// ---------------------------------------------------------------------------

/**
 * One ink per rarity, the same on every form: silver, teal, violet,
 * orange. Chosen apart from the suit inks (Eichel blue, Grün, Rot,
 * Schellen yellow) and light enough for the dark felt.
 */
export const RARITY_INK: Record<Rarity, { main: string; light: string; dark: string }> = {
  Common: { main: '#a9b4bc', light: '#eef2f4', dark: '#5f6a72' },
  Uncommon: { main: '#2ec4b6', light: '#a6f0e7', dark: '#16766d' },
  Rare: { main: '#a868ec', light: '#dec6ff', dark: '#5b2c9c' },
  Legendary: { main: '#ff8a1c', light: '#ffd49c', dark: '#a24e06' },
};

/** Number of pips per rarity (R3). */
const PIPS: Record<Rarity, number> = { Common: 1, Uncommon: 2, Rare: 3, Legendary: 4 };

/** Width of the rim band of R1. */
const BAND = 8;

function sparkle(x: number, y: number, edge: string): string {
  return `<path d="M${x - 1.5} ${y - 5}h3v3.5h3.5v3h-3.5v3.5h-3v-3.5h-3.5v-3h3.5z" fill="#fff8e0" stroke="${edge}" stroke-width=".9"/>`;
}

/** Pixel rows of the gem: k outline, X ink, H light, D dark. */
const GEM_ROWS = ['....k....', '...kXk...', '..kHXXk..', '.kHHXXXk.', 'kXHXXXXDk', '.kXXXXDk.', '..kXXDk..', '...kDk...', '....k....'];

function gem(x: number, y: number, rarity: Rarity): string {
  const ink = RARITY_INK[rarity];
  const fill: Record<string, string> = { k: OUTLINE, X: ink.main, H: ink.light, D: ink.dark };
  const cell = 2.8;
  const ox = x - (GEM_ROWS.length * cell) / 2;
  const oy = y - (GEM_ROWS.length * cell) / 2;
  let out = '';
  GEM_ROWS.forEach((row, j) =>
    [...row].forEach((ch, i) => {
      const f = fill[ch];
      if (f) out += `<rect x="${(ox + i * cell).toFixed(1)}" y="${(oy + j * cell).toFixed(1)}" width="${cell}" height="${cell}" fill="${f}"/>`;
    }),
  );
  if (rarity === 'Legendary') out += sparkle(x + 11, y - 10, ink.dark);
  return out;
}

function pips(form: Exclude<Form, 'Boss'>, rarity: Rarity): string {
  const spec = FORMS[form];
  const outer = formPts(form, 0);
  const top = spec.pips === 'top';
  const edgeY = extreme(outer, 0, top ? -1 : 1)[1];
  const n = PIPS[rarity];
  const size = 8;
  const gap = 3;
  const pad = 2.5;
  const w = n * size + (n - 1) * gap + 2 * pad;
  const h = size + 2 * pad;
  const cy = top ? edgeY + 3 : edgeY - 3;
  const x0 = CX - w / 2;
  const y0 = cy - h / 2;
  const ink = RARITY_INK[rarity];
  let out = `<rect x="${x0.toFixed(1)}" y="${y0.toFixed(1)}" width="${w.toFixed(1)}" height="${h}" rx="2" fill="${OUTLINE}"/>`;
  for (let i = 0; i < n; i++) {
    const x = x0 + pad + i * (size + gap);
    out += `<rect x="${x.toFixed(1)}" y="${(y0 + pad).toFixed(1)}" width="${size}" height="${size}" fill="${ink.main}"/>`;
    out += `<rect x="${x.toFixed(1)}" y="${(y0 + pad).toFixed(1)}" width="3" height="3" fill="${ink.light}"/>`;
  }
  return out;
}

/** Inside the paper: the cork of the label, or spirit, thick base and highlight of a glass. */
function glassMarkup(form: Exclude<Form, 'Boss'>, paperInset: number): string {
  const { glass: g, cork } = FORMS[form];
  const inner = formPts(form, paperInset);
  if (cork !== undefined) {
    const d = poly(clipPts(inner, 1, cork, false));
    return `<path d="${d}" fill="#b07a44"/><path d="${poly(clipBox(inner, 0, 0, 46, cork))}" fill="#d29a5c"/>`;
  }
  if (!g) return '';
  const bottom = g.base ?? 100;
  let out = `<path d="${poly(clipBox(inner, 0, g.liquid, 100, bottom))}" fill="${SPIRIT}"/>`;
  out += `<path d="${poly(clipPts(clipPts(inner, 1, g.liquid, true), 1, g.liquid + 2, false))}" fill="#e6d391"/>`;
  if (g.base !== undefined) out += `<path d="${poly(clipPts(inner, 1, g.base, true))}" fill="#a9cfdb"/>`;
  const [x0, y0, x1, y1] = g.shine;
  out += `<path d="${poly(clipBox(inner, x0, y0, x1, y1))}" fill="#ffffff" opacity=".75"/>`;
  return out;
}

// ---------------------------------------------------------------------------
// Mats
// ---------------------------------------------------------------------------

/** Shape and paper of a mat: a relic group, Schnapps, Item or Boss; HouseRule and Regular (older callers) mean a relic. */
export type CoasterKind = RelicGroup | RelicKind | 'Schnapps' | 'Item' | 'Boss';
/** A rarity, or the boss red. */
export type CoasterRing = Rarity | 'Boss';

const isGroup = (k: string): k is RelicGroup => k === 'Value' || k === 'Multiplier' || k === 'Control' || k === 'Tour';

/** The category of a relic: the group from the core, else the fallback by id, else Value. */
export function relicGroupOf(id: string, group?: RelicGroup | null): RelicGroup {
  return group ?? RELIC_GROUP[id] ?? 'Value';
}

/** The form an item is drawn in. */
export function formOf(id: string, kind: CoasterKind, opts: { group?: RelicGroup | null; schnapsStyle?: SchnapsStyle } = {}): Form {
  if (kind === 'Boss' || kind === 'Item') return kind;
  if (kind === 'Schnapps') return opts.schnapsStyle ?? EMBLEM_STYLE.schnaps;
  if (isGroup(kind)) return kind;
  return relicGroupOf(id, opts.group);
}

/** A small tilt between -5 and 5 degrees, fixed per id. */
export function tiltOf(id: string): number {
  let h = 2166136261;
  for (let i = 0; i < id.length; i++) h = Math.imul(h ^ id.charCodeAt(i), 16777619);
  return ((h >>> 0) % 11) - 5;
}

/** The motif of an item: its chosen variant, a boss motif or the generic motif of its kind. */
export function spriteFor(id: string, kind: CoasterKind, variant?: number): Sprite {
  if (kind === 'Boss') return BOSS_ART[id] ?? FALLBACK.Boss;
  const set = SHOP_ART[id];
  if (!set) return FALLBACK[kind === 'Schnapps' || kind === 'Item' || kind === 'Regular' ? kind : 'HouseRule'];
  return set[variant ?? CHOSEN[id] ?? 0] ?? set[0];
}

/** Whether an id has its own motif (not the generic one of its kind). */
export function hasOwnArt(id: string): boolean {
  return id in SHOP_ART || id in BOSS_ART;
}

export interface CoasterOpts {
  /** Width and height in CSS pixels. */
  size?: number;
  /** Accessible name; without it the picture is decorative. */
  label?: string;
  cls?: string;
  /** Force a motif variant (0 to 2) instead of CHOSEN. */
  variant?: number;
  /** Add the SVG namespace, for a file of its own. */
  standalone?: boolean;
  /** Category of a relic, overriding the one passed with the relic. */
  group?: RelicGroup | null;
  /** Override EMBLEM_STYLE.rarity. */
  rarityStyle?: RarityStyle;
  /** Override EMBLEM_STYLE.schnaps. */
  schnapsStyle?: SchnapsStyle;
}

function attr(text: string): string {
  return text.replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

function motifMarkup(sprite: Sprite, [x, y, s]: [number, number, number]): string {
  return `<g transform="translate(${x} ${y}) scale(${s})">${spriteMarkup(sprite)}</g>`;
}

/** The torn dark boss mat with its red dashed ring (unchanged from v1). */
function bossInner(sprite: Sprite): string {
  const d0 = formPath('Boss', 0);
  return (
    `<path d="${d0}" fill="#0b0c0e" transform="translate(0 4)"/>` +
    `<path d="${d0}" fill="#3a3330"/><path d="${d0}" fill="none" stroke="#1f1714" stroke-width="3"/>` +
    `<path d="${formPath('Boss', 5)}" fill="none" stroke="#bd3b32" stroke-width="4" stroke-dasharray="6 3"/>` +
    motifMarkup(sprite, [22, 20, 3.5])
  );
}

/** A mat of any form but Boss, with the rarity drawn in the given style. */
function matInner(form: Exclude<Form, 'Boss'>, rarity: Rarity, style: RarityStyle, sprite: Sprite): string {
  const spec = FORMS[form];
  const ink = RARITY_INK[rarity];
  const d0 = formPath(form, 0);
  const paperInset = style === 'Band' ? BAND : 0;
  let out = `<path d="${d0}" fill="#0b0c0e" transform="translate(0 4)"/>`;
  if (paperInset) {
    out += `<path d="${d0}" fill="${ink.main}" stroke="${OUTLINE}" stroke-width="2.5" stroke-linejoin="round"/>`;
    out += `<path d="${formPath(form, paperInset)}" fill="${spec.paper}" stroke="${OUTLINE}" stroke-width="1.5"/>`;
  } else {
    out += `<path d="${d0}" fill="${spec.paper}" stroke="${OUTLINE}" stroke-width="2.5" stroke-linejoin="round"/>`;
  }
  out += glassMarkup(form, paperInset);
  if (style === 'Gem') out += `<path d="${formPath(form, 5)}" fill="none" stroke="${ink.main}" stroke-width="3"/>`;
  if (style === 'Pips') out += `<path d="${formPath(form, 3)}" fill="none" stroke="${ink.main}" stroke-width="5"/>`;
  out += motifMarkup(sprite, spec.motif);
  if (style === 'Gem') {
    const [gx, gy] = spec.gem ?? extreme(formPts(form, 9), 1, -1);
    out += gem(gx, gy, rarity);
  } else if (style === 'Pips') {
    out += pips(form, rarity);
  } else if (rarity === 'Legendary') {
    const mid = formPts(form, BAND / 2);
    for (const [dx, dy] of [
      [0, -1],
      [1, 0],
      [0, 1],
      [-1, 0],
    ] as const) {
      const [x, y] = extreme(mid, dx, dy);
      out += sparkle(x, y, ink.dark);
    }
  }
  return out;
}

export interface InnerOpts {
  variant?: number;
  group?: RelicGroup | null;
  rarityStyle?: RarityStyle;
  schnapsStyle?: SchnapsStyle;
}

/** The inner drawing of a mat (in viewBox units), without the svg element. */
export function coasterInner(id: string, kind: CoasterKind, ring: CoasterRing, opts: InnerOpts = {}): string {
  const form = formOf(id, kind, opts);
  const sprite = spriteFor(id, kind, opts.variant);
  const body =
    form === 'Boss'
      ? bossInner(sprite)
      : matInner(form, ring === 'Boss' ? 'Common' : ring, opts.rarityStyle ?? EMBLEM_STYLE.rarity, sprite);
  return `<g transform="rotate(${tiltOf(id)} ${CX} ${CY})">${body}</g>`;
}

/** A mat as an SVG element. */
export function coasterSvg(id: string, kind: CoasterKind, ring: CoasterRing, opts: CoasterOpts = {}): string {
  const size = opts.size ?? 44;
  const a11y = opts.label ? `role="img" aria-label="${attr(opts.label)}"` : 'aria-hidden="true"';
  const ns = opts.standalone ? ' xmlns="http://www.w3.org/2000/svg"' : '';
  const data = `data-deckel="${attr(id)}" data-form="${formOf(id, kind, opts)}"`;
  return (
    `<svg${ns} class="deckel ${opts.cls ?? ''}" viewBox="0 0 100 100" width="${size}" height="${size}" shape-rendering="crispEdges" ${a11y} ${data}>` +
    `${coasterInner(id, kind, ring, opts)}</svg>`
  );
}

/** The mat of a relic, shaped by its category (group, else the fallback map). */
export function relicCoaster(
  relic: { id: string; kind?: RelicKind | null; rarity?: Rarity | null; group?: RelicGroup | null },
  opts: CoasterOpts = {},
): string {
  const group = relicGroupOf(relic.id, opts.group ?? relic.group);
  return coasterSvg(relic.id, group, relic.rarity ?? 'Common', opts);
}

/**
 * A Schnaps without any mat (user decision 03.11.2026): its 16 × 16 motif
 * enlarged in whole pixels, a small dot in the rarity ink at its foot on
 * the right. The size is rounded to a multiple of 20 so every motif pixel
 * is a whole number of screen pixels; the picture is 21 × 18 motif pixels
 * (the motif with its outline, the dot beside it).
 */
export function schnapsIcon(item: { id: string; rarity?: Rarity | null }, opts: CoasterOpts = {}): string {
  const unit = Math.max(1, Math.round((opts.size ?? 40) / 20));
  const ink = RARITY_INK[item.rarity ?? 'Common'];
  const a11y = opts.label ? `role="img" aria-label="${attr(opts.label)}"` : 'aria-hidden="true"';
  const ns = opts.standalone ? ' xmlns="http://www.w3.org/2000/svg"' : '';
  const sprite = spriteFor(item.id, 'Schnapps', opts.variant);
  return (
    `<svg${ns} class="deckel schnaps-pic ${opts.cls ?? ''}" viewBox="-1 -1 21 18" width="${unit * 21}" height="${unit * 18}" shape-rendering="crispEdges" ${a11y} data-deckel="${attr(item.id)}" data-form="Bare" data-rarity="${item.rarity ?? 'Common'}">` +
    `<g class="schnaps-motif">${spriteMarkup(sprite)}</g>` +
    `<g class="rarity-dot"><rect x="16" y="12" width="4" height="4" fill="${OUTLINE}"/><rect x="17" y="13" width="2" height="2" fill="${ink.main}"/><rect x="17" y="13" width="1" height="1" fill="${ink.light}"/></g></svg>`
  );
}

/** The drawing of a consumable: a Schnaps as its bare motif (schnapsIcon), another item on its mat. */
export function consumableCoaster(
  item: { id: string; category?: ConsumableCategory | null; rarity?: Rarity | null },
  opts: CoasterOpts = {},
): string {
  if (item.category === 'Schnapps') return schnapsIcon(item, opts);
  return coasterSvg(item.id, 'Item', item.rarity ?? 'Common', opts);
}

/** The mat of a boss. */
export function bossCoaster(id: string, opts: CoasterOpts = {}): string {
  return coasterSvg(id, 'Boss', 'Boss', opts);
}
