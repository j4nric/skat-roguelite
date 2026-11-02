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
  b: '#8a5a2b', // Eichel
  e: '#2e7d4f', // Grün
  r: '#bd3b32', // Rot
  R: '#e5776d', // light red
  y: '#b8840f', // Schellen
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
  bock_round: [
    sp('Ziegenkopf', `
      ..b..........b
      ..bb........bb
      ...bb......bb
      ....bwwwwwwb
      ...wwwwwwwwww
      ...wwkwwwwkww
      ....wwwwwwww
      .....wwwwww
      .....wwppww
      ......wwww
      ......wwww
      .......ww
      ......wwww
      .......ww`),
    sp('Ganze Ziege', `
      ..........b..b
      ...........bb
      ..........wwww
      ..........wkwwp
      .w.......wwwwww
      .wwwwwwwwwww.w
      .wwgwwwwwwww
      ..wwwgwwwww
      ..wwwwwwwww
      ..w.w....w.w
      ..w.w....w.w
      ..n.n....n.n`),
    sp('Bierkrug mit Hörnern', `
      b..........b
      bb........bb
      .bb.wwww.bb
      ..bwwwwwwb
      ..wwwwwwww
      ..aaaaaaaaggg
      ..ahaaaaaa..g
      ..ahaaaaaa..g
      ..ahaaaaaa..g
      ..aaaaaaaaggg
      ..aaaaaaaa
      ..gggggggg`),
  ],
  kontra: [
    sp('Sprechblase „Re!“', `
      ..wwwwwwwwwwww
      .wwwwwwwwwwwwww
      .wwrrwwrrrwrwww
      .wwrwrwrwwwrwww
      .wwrrwwrrwwrwww
      .wwrwrwrwwwwwww
      .wwrwrwrrrwrwww
      .wwwwwwwwwwwwww
      ..wwwwwwwwwwww
      ...www
      ..ww`),
    sp('Faust', `
      ..sssssss
      .sskskskss
      .sskskskss
      .sssssssss
      .ssssssssss
      .skkkkkksss
      .sssssssss
      ..sssssss
      ...sssss
      ...rrrrr
      ...rrrrr
      ...rrrrr`),
    sp('Mal zwei', `
      ..........rrrr
      .........rr..rr
      rr...rr......rr
      .rr.rr......rr
      ..rrr......rr
      .rr.rr....rr
      rr...rr..rr
      ........rrrrrrr`),
  ],
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
  ouvert_bonus: [
    sp('Drei offene Karten', `
      ...wwwwwwwwww
      ...wwwwwwwwww
      .wwkwwwrrwwwkww
      .wwkwwwrrwwwkww
      .eekwwwwwwwwkyy
      .eekwwwwwwwwkyy
      .wwkwwwwwwwwkww
      .wwkwwwwwwwwkww
      .wwkwwwwwwwwkww
      .wwkkkkkkkkkkww
      .ww..........ww
      .ww..........ww`),
    sp('Offenes Auge', `
      ..k..k...k..k
      .....wwwww
      ...wwwcccwww
      .wwwwcckccwwww
      wwwwcckkkccwwww
      .wwwwcckccwwww
      ...wwwcccwww
      .....wwwww`),
    sp('Lupe', `
      ..GGGGG
      .GhccccG
      GhccccccG
      GhccccccG
      GcccccccG
      .GcccccG
      ..GGGGG.b
      ........bb
      .........bb
      ..........bb
      ...........bb`),
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
  march_through: [
    sp('Trommel mit Schlegeln', `
      ..y..........y
      ...b........b
      ....b......b
      .....b....b
      ...wwwwwwwwww
      ..wwwwwwwwwwww
      ..yyyyyyyyyyyy
      ..wrrrrwwrrrrw
      ..rwrrwrrwrrwr
      ..rrwwrrrrwwrr
      ..rrrrrrrrrrrr
      ..yyyyyyyyyyyy`),
    sp('Marschstiefel', `
      ...bbbbb
      ...nnnnn
      ...nnnnn
      ...nnnnn
      ...nnnnn
      ...nnnnnn
      oo.nnnnnnnn
      ...nnnnnnnnnn
      oo.nnnnnnnnnnn
      ...kkkkkkkkkkk`),
    sp('Fanfare', `
      ..........YY
      .........YYY
      Y.......YYYY
      YYYYYYYYYYYY
      Y.......YYYY
      ...YY....YYY
      ...Y......YY`),
  ],
  blind_flight: [
    sp('Papierflieger', `
      .............w
      ...........wwg
      .........wwwgg
      .......wwwwwgg
      .....wwwwwwgg
      ...wwwwwwwwgg
      ..wwwwwwwwgg
      .......wwggg
      ...oo...wgg
      ........gg
      ..oo....g`),
    sp('Schlafmaske', `
      .vvvvv..vvvvv
      vvvvvvvvvvvvvv
      vkvvvkvvkvvvkv
      vvkkkvvvvkkkvv
      vvvvvvvvvvvvvv
      .vvvvv..vvvvv`),
    sp('Fledermaus', `
      ......G.G
      ......GGG
      .G...GRGRG...G
      GG..GGGGGGG..GG
      GGGGGGGGGGGGGGG
      GGGGGGGGGGGGGGG
      .GG.GGGGGGG.GG
      ..G..G.G.G..G`),
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
  granny: [
    sp('Oma mit Brille', `
      ......ggg
      .....ggggg
      ....gggggggg
      ...gggggggggg
      ...gssssssssg
      ...gkkksskkkg
      ...gkckkkkckg
      ...gkkksskkkg
      ...gpsssssspg
      ....ssskksss
      .....ssssss
      ....pppwwppp
      ...pppppppppp
      ...pppppppppp`),
    sp('Wollknäuel mit Nadeln', `
      .g.......g
      ..g.....g
      ...g...g
      ...rrrrrr
      ..rRrrrRrr
      .rrRrrrRrrr
      .rrrRrRrrrr
      .rrrRrRrrrr
      .rrRrrrRrrr
      ..rRrrrRrr
      ...rrrrrr.R
      ..........RR`),
    sp('Gugelhupf', `
      ....wwwww
      ..wwwwwwwww
      .wwwbwwwbwww
      .bwbbbwbbbwb
      bbbbbbbbbbbbb
      bObbObbObbObb
      bObbObbObbObb
      .bbbbbbbbbbb
      .ggggggggggg`),
  ],
  grandpa: [
    sp('Opa mit Mütze und Pfeife', `
      ....bbbbbbb
      ...bbbbbbbbbb
      ..nnnnnnnnnnnnn
      ...gssssssssg
      ...gskssssksg
      ...gsssppsssg
      ...wwwwwwwwww
      ....ssssssskkbb
      .....sssss...bb
      ....eeeeeee
      ...eeeeeeeee
      ..eeeeeeeeeee`),
    sp('Pfeife', `
      ..........g
      .........g
      ..........g
      .........g
      .......bbbb
      .......bnnb
      .......bbbb
      nnnnnnnbbb
      .......bb`),
    sp('Taschenuhr', `
      ....YY
      ...Y..Y
      ....YY
      ..YYYYYY
      .YwwwwwwY
      YwwwkwwwwY
      YwwwkwwwwY
      YwwwkkkwwY
      YwwwwwwwwY
      .YwwwwwwY
      ..YYYYYY`),
  ],
  referee: [
    sp('Gelbe und Rote Karte', `
      ....yyyyyy
      ....yyyyyy
      ....yyyrrrrrr
      ....yyyrrrrrr
      ....yyyrrrrrr
      ....yyyrrrrrr
      ....yyyrrrrrr
      ....yyyrrrrrr
      ....yyyrrrrrr
      .......rrrrrr
      .......rrrrrr`),
    sp('Trillerpfeife', `
      ..gggg
      .gggggggggg
      gghgggggggg
      gggggg
      gggggg
      .gggg
      ..rr
      .r..r
      ..rr`),
    sp('Schiri-Trikot', `
      ..wwkk..wwkk
      kkwwkkwwkkwwkk
      kkwwkkwwkkwwkk
      kkwwkkwwkkwwkk
      ..wwkkwwkkww
      ..wwkkwwkkww
      ..wwkkwwkkww
      ..wwkkwwkkww
      ..wwkkwwkkww
      ..wwkkwwkkww`),
  ],
  show_off: [
    sp('Angeber mit Sonnenbrille', `
      ......nnnnnn
      ....nnnnnnnnn
      ...nnnnnnnnnn
      ...nssssssssn
      ...kkkkkkkkkk
      ...hkkksshkkk
      ....ssssssss
      ....sswywwss
      .....ssssss
      ....rryssyrr
      ...rrrryyrrrr
      ..rrrrrrrrrrrr
      ..rrrrrrrrrrrr`),
    sp('Pfau', `
      ....eeeee
      ..eeveeevee
      .eveeeveeeve
      .eeeeeeeeeee
      eveeeccceeeve
      eeeeccckceeee
      ....ccccc
      .....ccc
      .....ccc
      ....cc.cc
      ....Y...Y`),
    sp('Pokal', `
      ..YYYYYYYYY
      Y.YwYYYYYYY.Y
      Y.YwYYYYYYY.Y
      .YYYYYYYYYYY
      ...YYYYYYY
      .....YYY
      ......Y
      ......Y
      ....YYYYY
      ...bbbbbbb
      ...bbbbbbb`),
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
  tuition: [
    sp('Doktorhut auf Groschen', `
      ......GGGG
      ....GGGGGGGG
      ..GGGGGGGGGGGG
      ....GGGGGGGGr
      .....kkkkkk.r
      .....kkkkkk.rr
      .....YYYYYY
      ....YYYYYYYY
      ....YYyyyyYY
      ....YYyyyyYY
      ....YYYYYYYY
      .....YYYYYY`),
    sp('Lehrbuch mit Groschen', `
      ..wwww...wwww
      .wwwwwwkwwwwww
      wGGGGwwkwwGGGGw
      wwwwwwwkwwwwwww
      wGGGwwwkwwYYYww
      wwwwwwwkwYyyYww
      wGGGGwwkwwYYYww
      wwwwwwwkwwwwwww
      bbbbbbbbbbbbbbb
      .bbbbbb.bbbbbb`),
    sp('Groschenstapel', `
      ..YYYYYYYY
      .yyyyyyyyyy
      .YYYYYYYYYY
      .yyyyyyyyyy
      .YYYYYYYYYY
      .yyyyyyyyyy
      .YYYYYYYYYY
      .yyyyyyyyyy
      .YYYYYYYYYY
      ..yyyyyyyy`),
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
  high_and_dry: [
    sp('Fisch auf dem Trockenen', `
      ...........YY
      ..........YYYY
      ...........YY
      ....ccccccc
      .c.ccccccckcc
      .ccccccccccccc
      .c.wwwwwwwwwc
      ....wwwwwwww
      .yyyyyyyyyyyyyy`),
    sp('Leeres Glas', `
      c......c
      chhhhhhc
      chhhhhhc
      chhhhhhc
      chhhhhhc
      .chhhhc
      .chhhhc
      .chaahc
      .cccccc`),
    sp('Kaktus in der Sonne', `
      .........YY
      ........YYYY
      ....e....YY
      ...eee
      .e.eee
      .e.eee.e
      .eeeee.e
      ...eeeeee
      ...eee
      ...eee
      yyyyyyyyyyy`),
  ],
  schneider_bock: [
    sp('Bock mit Maßband', `
      ..n..........n
      ..nn........nn
      ...nn......nn
      ....nllllllln
      ...llllllllll
      ...llkllllkll
      ....llllllll
      .....llllll
      .....llppll
      ....ykyykyyk
      ......llll
      .......ll
      ......llll
      .......ll`),
    sp('Ganzer Bock auf Maßband', `
      ..........n..n
      ...........nn
      ..........llll
      ..........lkllp
      .l.......llllll
      .lllllllllll.l
      .llblllllllll
      ..llllblllll
      ..lllllllll
      ..l.l....l.l
      ..l.l....l.l
      YkYYkYYkYYkYYkY`),
    sp('Maßband-Rolle', `
      ...YYYYY
      ..YYYYYYY
      .YYYkkkYYY
      .YYkgggkYY
      .YYkgkgkYY
      .YYkgggkYY
      .YYYkkkYYYYYYY
      ..YYYYYYYkYkYk
      ...YYYYYYYYYYY`),
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
  regular_guest: [
    sp('Stammkrug mit Deckel', `
      ...ggggggggg
      ..gggggggggg
      ...wwwwwwww
      ...wwwwwwwwggg
      ...GGGGGGGG..g
      ...wGwwGwwG..g
      ...wwwwwwww..g
      ...GGGGGGGG..g
      ...wwwwwwwwggg
      ...wwwwwwww
      ...wwwwwwww
      ..gggggggggg`),
    sp('Stammtisch-Schild', `
      G
      GGGGGGGGGGGG
      G..k......k
      G.bbbbbbbbbb
      G.bwwbwwbwwb
      G.bbbbbbbbbb
      G
      G
      G
      GGG`),
    sp('Gast mit Bier', `
      ...nnnn
      ..nnnnnn
      ..ssssss
      ..skssks
      ..ssssss
      ..snnnns
      ...ssss
      .GGGGGG.www
      GGGGGGGGaaag
      GGGGGGGGaaag
      GGGGGGGGaaa
      GGGGGGGG`),
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
  safety_net: [
    sp('Sicherheitsnetz', `
      ..b..........b
      ..brrrrrrrrrrb
      ..bw.w.w.w.w.b
      ..b.w.w.w.w.wb
      ..bw.w.w.w.w.b
      ..b.w.w.w.w.wb
      ..brw.w.w.w.rb
      ..b.rrrrrrrr.b
      ..b..........b
      .bbb........bbb`),
    sp('Fallschirm', `
      ....rrrrrr
      ..rrwwrrwwrr
      .rrwwwrrwwwrr
      rrwwwwrrwwwwrr
      .k....kk....k
      ..k...kk...k
      ...k..kk..k
      ....k.kk.k
      .....bbbb
      .....bbbb`),
    sp('Kiste mit doppeltem Boden', `
      bbbbbbbbbbbb
      bddddddddddb
      bdwwddrrdddb
      bdwwddrrdddb
      bddddddddddb
      bbbbbbbbbbbb
      bdYYddYYdddb
      bbbbbbbbbbbb`),
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
  glass_house: [
    sp('Glashaus', `
      .......gg
      ......gccg
      .....gccccg
      ....gccccccg
      ...gccccccccg
      ..gggggggggggg
      ..gchccggchccg
      ..gcchcggcchcg
      ..gccccggccccg
      ..gggggggggggg
      ..gccccggccccg
      ..gcckcggccccg
      ..gggggggggggg`),
    sp('Fenster mit Sprung', `
      GGGGGGGGGGG
      GhcccGhcccG
      GccccGcckcG
      GccccGckccG
      GGGGGGGGGGG
      GhcccGckkcG
      GccccGkcccG
      GccccGccccG
      GGGGGGGGGGG`),
    sp('Glasglocke über einer Karte', `
      ....cccc
      ..cccccccc
      .chcccccccc
      .chccwwwccc
      chcccwrwcccc
      chcccwrwcccc
      cccccwwwcccc
      gggggggggggg`),
  ],
  tally_stick: [
    sp('Kerbholz mit Messer', `
      .ll.ll.ll.ll.ll
      .bbkbbkbbkbbkbb
      .bbbbbbbbbbbbbb
      .bbbbbbbbbbbbbb
      .nnnnnnnnnnnnnn
      ...............
      ...............
      ...gggggg
      ..gggggggbbbbb`),
    sp('Gespaltenes Kerbholz', `
      .lbb.bbl
      .lbb.bbl
      .kbb.bbk
      .lbb.bbl
      .kbb.bbk
      .lbb.bbl
      .kbb.bbk
      .lbb.bbl
      .kbb.bbk
      .lbb.bbl
      .lbb.bbl
      .nnn.nnn`),
    sp('Steintafel mit Kerben', `
      .gggggggggg
      gggggggggggg
      ggGgGgGgGggg
      ggGgGgGGGggg
      ggGgGGGgGggg
      ggGGGgGgGggg
      ggGgGgGgGggg
      gggggggggggg
      .gggggggggg`),
  ],
  with_four: [
    sp('Große Vier mit Farben', `
      .bb..........ee
      .bb..GG..GG..ee
      .....GG..GG
      .....GG..GG
      .....GG..GG
      .....GGGGGGG
      .....GGGGGGG
      .........GG
      .........GG
      .........GG
      .rr......GG..yy
      .rr..........yy`),
    sp('Vier Unter', `
      bbb.eee.rrr.yyy
      bbb.eee.rrr.yyy
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
  seven_sleeper: [
    sp('Siebenschläfer', `
      ..........GGGG
      ............G
      ...........G
      ..........GGGG
      ...gg
      ..gpgggggggg
      .ggggggggggggg
      .gkkgggggggwwgg
      .pgggggggggwwgg
      ..gggggggggwwgg
      ...gggggggwwwg
      .....wwwwwwww`),
    sp('Bett mit Schläfer', `
      ...........GGG
      ............G
      b..........GGG
      b
      bwwss
      bwwssoooooooob
      bbooooooooooob
      bbbbbbbbbbbbbb
      b............b`),
    sp('Mond und Sterne', `
      ....YYY
      ..YYYY....Y
      .YYY.....YYY
      .YY.......Y
      YYY
      YYY......Y
      YYY.....YYY
      .YY......Y
      .YYY
      ..YYYY
      ....YYY`),
  ],
  clean_slate: [
    sp('Weiße Weste', `
      ...ww......ww
      ...www....www
      ..wwww....wwww
      ..wwwww..wwwww
      ..wwwwwkywwwww
      ..wwwwwkwwwwww
      ..wwwwwkywwwww
      ..wgggwkwgggww
      ..wwwwwkywwwww
      ..wwwwwkwwwwww
      ..wwwww..wwwww
      ..www......www`),
    sp('Seife mit Blasen', `
      ........hh
      .......hhhh
      ..hh....hh
      .hhhh
      ..hh..hh
      ...pppppppp
      ..pppppppppp
      .pphhppppppp
      .pppppppppp
      ..pppppppp`),
    sp('Leere Tafel', `
      bbbbbbbbbbbb
      bMMMMMMMMwMb
      bMMMMMMMwwwb
      bMMMMMMMMwMb
      bMMMMMMMMMMb
      bMMMMMMMMMMb
      bMMMMMMMMMMb
      bMMMMMMMMMMb
      bbbbbbbbbbbb
      ...YYYY`),
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
  old_deck: [
    sp('Vergilbte Karte mit Eselsohr', `
      YYYYYYY
      YfffffYY
      YfnfffYfY
      YfffffYYYY
      YffffffffY
      YffnfffffY
      YffffffnfY
      YffffffffY
      YfnffffffY
      YffffffffY
      YYYYYYYYYY`),
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
  changeling: [
    sp('Halb grünes, halb violettes Gesicht', `
      ...eeevvv
      ..eeeevvvv
      .eeeeevvvvv
      .eekeeevkvv
      .eeeeevvvvv
      .eeeeevvvvv
      .eekkkkkkvv
      ..eeeevvvv
      ...eeevvv`),
  ],
  jack_parade: [
    sp('Vier Unter in einer Reihe', `
      .G...G...G...G.
      GGG.GGG.GGG.GGG
      sss.sss.sss.sss
      bbb.eee.rrr.yyy
      bbb.eee.rrr.yyy
      bbb.eee.rrr.yyy
      b.b.e.e.r.r.y.y`),
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
  beer: [
    sp('Bierkrug', `
      ....ww..ww
      ...wwwwwwwww
      ..wwwwwwwwww
      ...aaaaaaaa
      ...ahaaaaaaccc
      ...ahaaaaaac..c
      ...ahaaaaaac..c
      ...ahaaaaaac..c
      ...aaaaaaaaccc
      ...aaaaaYaa
      ...aaaaaaaa
      ...cccccccc`),
    sp('Bierflasche', `
      ..YY
      ..nn
      ..nn
      .bbbb
      bbbbbb
      bwwwwb
      bwrrwb
      bwwwwb
      bbbbbb
      bbbbbb
      bbbbbb`),
    sp('Pilstulpe', `
      ..wwwwww
      .wwwwwwww
      .aaaaaaaa
      .ahaaaaaa
      ..ahaaaa
      ...aaaa
      ....aa
      ....cc
      ....cc
      ..cccccc`),
  ],
  new_deal: [
    sp('Karte mit Pfeilen', `
      .....oooooo
      ...oo......oo
      ..o..wwwwwwooo
      ..o..wwwwww.o
      .....wwrrww
      .....wrrrrw
      .....wrrrrw
      .....wwrrww
      .....wwwwww
      ...o.wwwwww..o
      ..ooowwwwww..o
      ...oo......oo
      .....oooooo`),
    sp('Kartenfächer', `
      ....wwwwww
      ....wrrrrw
      wwwwwrrrrwwwww
      wrrrwrYYrwrrrw
      wrrrwrYYrwrrrw
      wrYrwrrrrwrYrw
      wrrrwrrrrwrrrw
      wrrrwwwwwwrrrw
      wrrrw....wrrrw
      wwwww....wwwww`),
    sp('Austeilende Hand', `
      ..........www
      ..........rrw
      .......www
      .......rrw
      ....www
      ....rrw
      sss
      sssss
      ssssss
      sssss
      rr`),
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
  bitters: [
    sp('Bauchige braune Flasche', `
      .......ll
      .......nn
      .......nn
      ......nnnn
      ....nnnnnnnn
      ...nbnnnnnnnn
      ...nrrrrrrrrn
      ...nrwwwwwwrn
      ...nrrrrrrrrn
      ...nnnnnnnnnn
      ...nbnnnnnnnn
      ....nnnnnnnn`),
    sp('Gläschen Magenbitter', `
      c......c
      chnnnnnc
      chnnnnnc
      .cnnnnc
      .cnnnnc
      ..cnnc
      ...cc
      ...cc
      .cccccc`),
    sp('Apothekerflasche', `
      ...lll
      ...lll
      ...nnn
      ..nnnnn
      .nnnnnnn
      nwwwwwwwn
      nwwwrwwwn
      nwwrrrwwn
      nwwwrwwwn
      nwwwwwwwn
      nnnnnnnnn
      .nnnnnnn`),
  ],
  chalk: [
    sp('Kreidestück', `
      ...........www
      ..........www
      .........www
      ........www
      .......www
      ......www
      .....www
      ....ggg
      ...g.g
      ..........
      ...wwwwww
      ..........
      .g`),
    sp('Kreideschachtel', `
      .ww.ww.ww
      .ww.ww.ww
      oooooooooo
      owwwwwwwwo
      oooooooooo
      oooooooooo
      oooooooooo`),
    sp('Umgemalte Karte', `
      wwwwwwwwwww
      wwwwwwwwwww
      wrrwwwoweew
      wrrwooooeew
      wrrwwwoweew
      wwwwwwwwwww
      wwwwwwwwwww
      wwwwwwwwwww`),
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
  bock_round: 1,
  kontra: 0,
  last_trick_seven: 1,
  concede: 0,
  ouvert_bonus: 1,
  skat_redraw: 2,
  virgin: 0,
  old_lady: 0,
  tour: 1,
  march_through: 2,
  blind_flight: 2,
  kibitzer: 0,
  landlord: 0,
  granny: 0,
  grandpa: 0,
  referee: 2,
  show_off: 0,
  card_counter: 2,
  every_bit_counts: 0,
  blind_eye: 1,
  tuition: 0,
  beer_mat: 0,
  high_and_dry: 0,
  schneider_bock: 1,
  lucky_streak: 0,
  tailor: 0,
  regular_guest: 2,
  null_eight_fifteen: 0,
  safety_net: 0,
  fat_sow: 1,
  glass_house: 0,
  tally_stick: 1,
  with_four: 1,
  seven_sleeper: 1,
  clean_slate: 2,
  fresh_deck: 0,
  ink_pad: 0,
  beer: 0,
  new_deal: 1,
  shot: 0,
  double_grain: 1,
  herbal: 0,
  bitters: 0,
  chalk: 1,
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
