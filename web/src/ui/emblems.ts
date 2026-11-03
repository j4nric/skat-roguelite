// Bierdeckel emblems (style "R1 Bierdeckel", variant "B3 Form und
// Pappfarbe"): every relic, consumable and boss is a small beer mat with
// its own 16 × 16 pixel motif. The kind picks shape and paper colour
// (house rule round and cream, Stammtisch figure rounded square in
// blue-grey, Schnaps with a crown-cork edge in mint, other items octagonal
// in ochre, boss torn and dark), the rarity a dashed ring along the edge.
// Each mat is tilted a little, by its id, and casts a hard shadow.
//
// Sprites are rows of palette keys ('.' is empty); a dark outline is drawn
// around every motif. Rows may be ragged: the motif is centred by its
// bounding box. Relics and consumables have three motif variants each, the
// one shown is picked in CHOSEN; bosses have one. Unknown ids get a
// generic motif of their kind. Pure presentation, no game logic; this
// module has no runtime imports so scripts can load it directly.

import type { ConsumableCategory, Rarity, RelicKind } from '../types';

/** Shape and paper of a mat. */
export type CoasterKind = 'HouseRule' | 'Regular' | 'Schnapps' | 'Item' | 'Boss';
/** Colour of the dashed ring: a rarity, or the boss red. */
export type CoasterRing = Rarity | 'Boss' | 'Legendary';

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
  // ---- Item review 02.11.2026 (one motif each, not voted)
  closing_time: [
    sp('Mondsichel mit Stern über der Kneipentür', `
      .....YYY......Y
      ...YYYY......YYY
      ..YYY.........Y
      ..YY
      ..YY.....nnnnnn
      ..YYY....nYYYYn
      ...YYYY..nYYYYn
      .....YYY.nnnnnn
      .........nbbbbn
      .........nbbYbn
      .........nbbbbn
      .........nbbbbn`),
  ],
  season_ticket: [
    sp('Fahrkarte mit Lochung', `
      mmmmmmmmmmmmmm
      mMMMMMMMMMMMMm
      mwwwwwwwwwwwwm
      mwGGGGww.wwwwm
      mwwwwwwwwwwwwm
      mwGGGwwwGGGwwm
      mwwwwwwwwwwwwm
      mwGGGGGwwwwwwm
      mwwwwwwwwwwwwm
      mmmmmmmmmmmmmm`),
  ],
  kitty: [
    sp('Blechkasse mit Münzschlitz', `
      ....YYYY
      ....YyyY
      ..GGGGGGGGGG
      .GggggggggggG
      .GgkkkkkkkkgG
      .GggggggggggG
      .GgGGGGGGGGgG
      .GgGyyyyyyGgG
      .GgGyYYYYyGgG
      .GgGyyyyyyGgG
      .GggggggggggG
      ..GGGGGGGGGG`),
  ],
  slate: [
    sp('Bierdeckel mit Kreidestrichen', `
      ....wwwwww
      ..wwwwwwwwww
      .wwGwGwGwGwww
      .wwGwGwGwGwww
      wwwGwGwGwGwwww
      wwwGGGGGGGGwww
      wwwGwGwGwGwwww
      .wwGwGwGwGwww
      .wwwwwwwwwwww
      ..wwwwwwwwww
      ....wwwwww`),
  ],
  guardian_angel: [
    sp('Heiligenschein mit Flügeln', `
      .....YYYYYY
      ....Y......Y
      .....YYYYYY
      ......ssss
      hh....ssss....hh
      hhh..wwwwww..hhh
      hhhhwwwwwwwwhhhh
      .hhhwwwwwwwwhhh
      ..hhwwwwwwwwhh
      ....wwwwwwww
      ....wwwwwwww`),
  ],
  dive_expert: [
    sp('Lupe über einer dunklen Kellertür', `
      ..dddddd
      .dddddddd
      .ddaddddd
      .dddddddd....GG
      .dddddddd..GGccGG
      .dddddddd.GcccccG
      .dddddddd.GcchccG
      .dddddddd.GcccccG
      ...........GGGGG
      ..........bb
      .........bb
      ........bb`),
  ],
  counter_beer: [
    sp('Bierkrug mit Schaumkrone', `
      ..wwwwwwww
      .wwwwwwwwww
      .waaaaaaaaw
      .aaaaaaaaaaGGG
      .aaaaaaaaaa..G
      .aafaaaaaaa..G
      .aafaaaaaaa..G
      .aafaaaaaaa..G
      .aaaaaaaaaaGGG
      .aaaaaaaaaa
      .GGGGGGGGGG`),
  ],
  snifter: [
    sp('Cognacschwenker', `
      ...cccccccc
      ..c........c
      .c..........c
      .c..........c
      .cOOOOOOOOOOc
      .cOOlOOOOOOOc
      ..cOOOOOOOOc
      ...cOOOOOOc
      .....cccc
      ......cc
      ......cc
      ...cccccccc`),
  ],
  road_beer: [
    sp('Bierflasche für unterwegs', `
      ......gg
      ......nn
      ......nn
      .....nnnn
      ....nnnnnn
      ....nwwwwn
      ....nwrrwn
      ....nwwwwn
      ....nnnnnn
      ....nnnnnn
      ....nnnnnn
      .....nnnn`),
  ],
  // ---- consumables (core/content/consumables.ron)
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
    sp('Blaues Cocktailglas mit Zitronenscheibe', `
      ..............LL
      cccccccccccccLLL
      .cJJJJJJJJJJJcL
      ..cJJhJJJJJJc
      ...cJJJJJJJc
      ....cJJJJJc
      .....cJJJc
      ......cJc
      .......c
      .......c
      .......c
      ....ccccccc`),
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
  kontra_hans: sp('Wütender Schreihals', `
    ...n.n.nn.n.n
    ...nnnnnnnnnn
    ...nRRRRRRRRn.y
    ...RkkRRRRkkR.y
    ...RRwkRRkwRR.y
    ...RRRRRRRRRR
    ...RRRkkkkRRR
    ...RRkrrrrkRR.y
    ...RRRkkkkRRR
    ....RRRRRRRR
    ......ssss
    ...GGGGGGGGGG
    ..GGGGGGGGGGGG`),
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
  revolutionary: sp('Rote Mütze mit Stern', `
    .......r
    ...rrrrrrrrr
    ..rrrrrYrrrrr
    ..rrrrYYYrrrrr
    ....ssssssss
    ....skssssks
    ....ssssssss
    ....nnnnnnnn
    ....nnnnnnnn
    ....nnnnnnnn
    .....nnnnnn
    ..eeeeeeeeeeee
    ..eeeeeeeeeeee`),
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
  kibitz_killer: sp('Durchgestrichenes Auge', `
    .............rr
    ............rr
    .....wwwwwwrr
    ...wwwGGGGrrw
    ..wwwGGkkrrwww
    .wwwwGkkrrGwwww
    ..wwwGGrrGGwww
    ...wwwrrGGwww
    .....rrwwww
    ....rr
    ...rr
    ..rr`),
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
const FALLBACK: Record<CoasterKind, Sprite> = {
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
  blue_curacao: 0,
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
// Coaster shapes (viewBox 0 0 100 100, centre 50 48)
// ---------------------------------------------------------------------------

const CX = 50;
const CY = 48;

function poly(points: [number, number][]): string {
  return 'M' + points.map(([x, y]) => `${x.toFixed(1)} ${y.toFixed(1)}`).join('L') + 'Z';
}

function radial(n: number, radius: (k: number, a: number) => number, phase = 0): string {
  const pts: [number, number][] = [];
  for (let k = 0; k < n; k++) {
    const a = phase + (k * 2 * Math.PI) / n;
    const r = radius(k, a);
    pts.push([CX + r * Math.cos(a), CY + r * Math.sin(a)]);
  }
  return poly(pts);
}

/** Fixed jitter of the torn boss edge (a small seeded generator, looks only). */
const TORN_JITTER: number[] = (() => {
  let seed = 7;
  return Array.from({ length: 48 }, () => {
    seed = (seed * 1103515245 + 12345) % 2147483648;
    return -3 + (seed / 2147483648) * 5;
  });
})();

function roundedSquare(i: number): string {
  const x = 6 + i;
  const y = 4 + i;
  const w = 88 - 2 * i;
  const r = Math.max(4, 16 - i);
  return `M${x + r} ${y}H${x + w - r}Q${x + w} ${y} ${x + w} ${y + r}V${y + w - r}Q${x + w} ${y + w} ${x + w - r} ${y + w}H${x + r}Q${x} ${y + w} ${x} ${y + w - r}V${y + r}Q${x} ${y} ${x + r} ${y}Z`;
}

const SHAPES: Record<CoasterKind, (inset: number) => string> = {
  HouseRule: (i) => radial(72, () => 44 - i),
  Regular: roundedSquare,
  Schnapps: (i) => radial(140, (_, a) => 41.5 - i + 3 * Math.cos(14 * a)),
  Item: (i) => radial(8, () => 47 - i, Math.PI / 8),
  Boss: (i) => radial(48, (k) => 43 - i + (i === 0 ? (TORN_JITTER[k] ?? 0) : 0)),
};

const shapeCache = new Map<string, string>();
function shape(kind: CoasterKind, inset: number): string {
  const key = `${kind}:${inset}`;
  let d = shapeCache.get(key);
  if (d === undefined) shapeCache.set(key, (d = SHAPES[kind](inset)));
  return d;
}

const PAPER: Record<CoasterKind, string> = {
  HouseRule: '#ecebe4',
  Regular: '#d6e2ec',
  Schnapps: '#d8ebdf',
  Item: '#f1dfb8',
  Boss: '#3a3330',
};

const RING: Record<string, string> = {
  Common: '#a4adb3',
  Uncommon: '#8fc7a8',
  Rare: '#d1773f',
  Boss: '#bd3b32',
  Legendary: '#e3b24c',
};

/**
 * Legendär: a second, solid gold ring inside the dashed one and four
 * pixel sparkles on the rim, so the tier reads at a glance next to Rare.
 */
function legendaryRing(kind: CoasterKind): string {
  const sparkles = [
    [CX, CY - 45],
    [CX + 45, CY],
    [CX, CY + 45],
    [CX - 45, CY],
  ]
    .map(([x = 0, y = 0]) => `<path d="M${x - 1.5} ${y - 5}h3v3.5h3.5v3h-3.5v3.5h-3v-3.5h-3.5v-3h3.5z" fill="#fff4c8" stroke="#8a6512" stroke-width=".8"/>`)
    .join('');
  return `<path d="${shape(kind, 10)}" fill="none" stroke="#e3b24c" stroke-width="2"/>${sparkles}`;
}

/** A small tilt between -5 and 5 degrees, fixed per id. */
export function tiltOf(id: string): number {
  let h = 2166136261;
  for (let i = 0; i < id.length; i++) h = Math.imul(h ^ id.charCodeAt(i), 16777619);
  return ((h >>> 0) % 11) - 5;
}

/** The motif of an item: its chosen variant, a boss motif or the fallback of its kind. */
export function spriteFor(id: string, kind: CoasterKind, variant?: number): Sprite {
  if (kind === 'Boss') return BOSS_ART[id] ?? FALLBACK.Boss;
  const set = SHOP_ART[id];
  if (!set) return FALLBACK[kind];
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
}

function attr(text: string): string {
  return text.replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

/** The inner drawing of a mat (in viewBox units), without the svg element. */
export function coasterInner(id: string, kind: CoasterKind, ring: CoasterRing, variant?: number): string {
  const d0 = shape(kind, 0);
  const rim = kind === 'Boss' ? `<path d="${d0}" fill="none" stroke="#1f1714" stroke-width="3"/>` : '';
  return (
    `<g transform="rotate(${tiltOf(id)} ${CX} ${CY})">` +
    `<path d="${d0}" fill="#0b0c0e" transform="translate(0 4)"/>` +
    `<path d="${d0}" fill="${PAPER[kind]}"/>${rim}` +
    `<path d="${shape(kind, 5)}" fill="none" stroke="${RING[ring] ?? RING.Common}" stroke-width="4" stroke-dasharray="6 3"/>` +
    (ring === 'Legendary' ? legendaryRing(kind) : '') +
    `<g transform="translate(22 20) scale(3.5)">${spriteMarkup(spriteFor(id, kind, variant))}</g></g>`
  );
}

/** A beer mat as an SVG element. */
export function coasterSvg(id: string, kind: CoasterKind, ring: CoasterRing, opts: CoasterOpts = {}): string {
  const size = opts.size ?? 44;
  const a11y = opts.label ? `role="img" aria-label="${attr(opts.label)}"` : 'aria-hidden="true"';
  const ns = opts.standalone ? ' xmlns="http://www.w3.org/2000/svg"' : '';
  return `<svg${ns} class="deckel ${opts.cls ?? ''}" viewBox="0 0 100 100" width="${size}" height="${size}" shape-rendering="crispEdges" ${a11y} data-deckel="${attr(id)}">${coasterInner(id, kind, ring, opts.variant)}</svg>`;
}

/** The mat of a relic: house rule or Stammtisch figure. */
export function relicCoaster(relic: { id: string; kind?: RelicKind | null; rarity?: Rarity | null }, opts: CoasterOpts = {}): string {
  return coasterSvg(relic.id, relic.kind === 'Regular' ? 'Regular' : 'HouseRule', relic.rarity ?? 'Common', opts);
}

/** The mat of a consumable: a Schnaps or another item. */
export function consumableCoaster(
  item: { id: string; category?: ConsumableCategory | null; rarity?: Rarity | null },
  opts: CoasterOpts = {},
): string {
  return coasterSvg(item.id, item.category === 'Schnapps' ? 'Schnapps' : 'Item', item.rarity ?? 'Common', opts);
}

/** The mat of a boss. */
export function bossCoaster(id: string, opts: CoasterOpts = {}): string {
  return coasterSvg(id, 'Boss', 'Boss', opts);
}
