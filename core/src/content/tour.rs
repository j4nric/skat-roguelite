//! Content of the Kneipentour (ADR 0016, DESIGN 5): the word lists of the
//! city's name generator and the maluses of a Spelunke, from `tour.ron`.
//!
//! Names are presentation: they never change play. The generator is
//! deterministic ([`CityNames::generate`]), so equal seeds name equal
//! cities. A Spelunke malus is a rule of one pub only; what it does is
//! data: table rules as passive [`Effect`]s (fired like a boss rule, as
//! [`super::Source::Malus`]) and three run restrictions that are no table
//! rule (no Einmischen, Schnäpse locked, cards cursed for this pub only).

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use super::{CardModId, CharacterPool, Effect};
use crate::rng::Rng;

content_id!(
    /// Id of a Spelunke malus ([`MalusDef`]).
    MalusId
);

/// A name pattern: text with `{list}` placeholders, each filled with a
/// word of that list of [`CityNames::words`], and its draw weight.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamePattern {
    /// The pattern, e.g. `"Zum {adj_m} {masc}"`.
    pub pattern: String,
    /// Weight when drawn (default 1).
    #[serde(default = "super::one")]
    pub weight: u32,
}

/// How a district looks on the city map (Ansicht des Stadtteils): pure
/// presentation, it never changes play.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub enum DistrictLook {
    /// Old lanes and a river (Altstadt).
    #[default]
    OldTown,
    /// Water and quays (Hafen).
    Harbour,
    /// Tracks and a station hall (Bahnhof).
    Station,
}

/// A district name of the generator with the look it gets on the map.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistrictName {
    /// The name, e.g. `"Altstadt"`.
    pub name: String,
    /// Its look on the map.
    #[serde(default)]
    pub look: DistrictLook,
}

/// A named district of a run's city: name, look and the names of its
/// pubs (the last the boss pub).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CityDistrict {
    /// The district's name.
    pub name: String,
    /// Its look on the map.
    pub look: DistrictLook,
    /// Names of its pubs, the boss pub last.
    pub pubs: Vec<String>,
}

/// The word lists of the name generator (German).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CityNames {
    /// Names of the districts (Stadtteile) with their looks; a run draws
    /// distinct ones, of different looks while there are enough looks.
    pub districts: Vec<DistrictName>,
    /// Patterns of a normal pub (Kneipe).
    pub pubs: Vec<NamePattern>,
    /// Patterns of a boss pub (Boss-Kneipe).
    pub boss_pubs: Vec<NamePattern>,
    /// Patterns of a Spelunke.
    pub spelunken: Vec<NamePattern>,
    /// Word lists by name, used by the patterns.
    pub words: BTreeMap<String, Vec<String>>,
}

/// How many tries the generator makes for a name not used yet before it
/// takes a repeated one.
const NAME_TRIES: u32 = 12;

impl CityNames {
    /// Fill one pattern drawn from `patterns` with words; `None` without
    /// patterns.
    fn fill(&self, rng: &mut Rng, patterns: &[NamePattern]) -> Option<String> {
        let total: u64 = patterns.iter().map(|p| u64::from(p.weight)).sum();
        if total == 0 {
            return None;
        }
        let mut roll = rng.gen_range(0..total);
        let pattern = patterns.iter().find(|p| {
            if roll < u64::from(p.weight) {
                true
            } else {
                roll -= u64::from(p.weight);
                false
            }
        })?;
        let mut out = String::new();
        let mut rest = pattern.pattern.as_str();
        while let Some(start) = rest.find('{') {
            out.push_str(&rest[..start]);
            let Some(len) = rest[start..].find('}') else {
                break;
            };
            let key = &rest[start + 1..start + len];
            let words = self.words.get(key).map_or(&[][..], Vec::as_slice);
            if !words.is_empty() {
                let pick = rng.gen_range(0..words.len() as u64);
                out.push_str(&words[usize::try_from(pick).unwrap_or(0)]);
            }
            rest = &rest[start + len + 1..];
        }
        out.push_str(rest);
        Some(out)
    }

    /// A name from `patterns` that is not in `used` if the generator finds
    /// one within a few tries; added to `used`.
    pub fn fresh(&self, rng: &mut Rng, patterns: &[NamePattern], used: &mut Vec<String>) -> String {
        let mut name = String::new();
        for _ in 0..NAME_TRIES {
            name = self.fill(rng, patterns).unwrap_or_default();
            if !used.contains(&name) {
                break;
            }
        }
        used.push(name.clone());
        name
    }

    /// Name a city of `districts` districts with `pubs` pubs each (the
    /// last of each a boss pub), all from one generator seeded with
    /// `seed`: the looks present in the list in a seeded order, one per
    /// district (repeated only when there are more districts than looks),
    /// then per district an unused name of its look (any unused name if
    /// its look has none left, a numbered one if the list is used up) and
    /// distinct pub names.
    #[must_use]
    pub fn generate(&self, seed: u64, districts: u32, pubs: u32) -> Vec<CityDistrict> {
        let mut rng = Rng::from_seed(seed);
        let mut looks: Vec<DistrictLook> = self.districts.iter().map(|d| d.look).collect();
        looks.sort_unstable();
        looks.dedup();
        // Seeded Fisher-Yates shuffle of the looks.
        for i in (1..looks.len()).rev() {
            let j = rng.gen_range(0..(i as u64 + 1));
            looks.swap(i, usize::try_from(j).unwrap_or(0));
        }
        let mut pool: Vec<&DistrictName> = self.districts.iter().collect();
        let mut used = Vec::new();
        (0..districts)
            .map(|d| {
                let wanted = (!looks.is_empty())
                    .then(|| looks[usize::try_from(d).unwrap_or(0) % looks.len()]);
                let of_look: Vec<usize> = (0..pool.len())
                    .filter(|&i| Some(pool[i].look) == wanted)
                    .collect();
                let candidates: Vec<usize> = if of_look.is_empty() {
                    (0..pool.len()).collect()
                } else {
                    of_look
                };
                let (name, look) = if candidates.is_empty() {
                    (format!("Viertel {}", d + 1), wanted.unwrap_or_default())
                } else {
                    let pick = rng.gen_range(0..candidates.len() as u64);
                    let chosen = pool.remove(candidates[usize::try_from(pick).unwrap_or(0)]);
                    (chosen.name.clone(), chosen.look)
                };
                let names = (0..pubs)
                    .map(|p| {
                        let patterns = if p + 1 == pubs {
                            &self.boss_pubs
                        } else {
                            &self.pubs
                        };
                        self.fresh(&mut rng, patterns, &mut used)
                    })
                    .collect();
                CityDistrict {
                    name,
                    look,
                    pubs: names,
                }
            })
            .collect()
    }
}

/// Cards of the player's edition cursed for one pub only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MalusCurse {
    /// The curse.
    pub card_mod: CardModId,
    /// Its value.
    #[serde(default = "super::one_i32")]
    pub value: i32,
    /// How many random cards without modification get it.
    pub cards: u32,
}

/// A Spelunke malus (ADR 0016): one rule of a single pub, shown before
/// the route is chosen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MalusDef {
    /// Unique id.
    pub id: MalusId,
    /// German display name.
    pub name: String,
    /// German description.
    pub text: String,
    /// Weight when drawn (default 1).
    #[serde(default = "super::one")]
    pub weight: u32,
    /// Characters that meet it (empty: all).
    #[serde(default)]
    pub characters: CharacterPool,
    /// Table rules during the pub, like a boss rule.
    #[serde(default)]
    pub passive: Vec<Effect>,
    /// No Einmischen in this pub.
    #[serde(default)]
    pub no_redeal: bool,
    /// Schnäpse cannot be used in this pub.
    #[serde(default)]
    pub lock_schnapps: bool,
    /// Cards cursed for this pub only.
    #[serde(default)]
    pub curse: Option<MalusCurse>,
}

/// The content of `tour.ron`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TourContent {
    /// The name generator.
    pub city: CityNames,
    /// The Spelunke maluses.
    pub maluses: Vec<MalusDef>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn district(name: &str, look: DistrictLook) -> DistrictName {
        DistrictName {
            name: name.to_owned(),
            look,
        }
    }

    fn names() -> CityNames {
        CityNames {
            districts: vec![
                district("Altstadt", DistrictLook::OldTown),
                district("Schlossberg", DistrictLook::OldTown),
                district("Unterstadt", DistrictLook::OldTown),
                district("Hafen", DistrictLook::Harbour),
                district("Bahnhofsviertel", DistrictLook::Station),
                district("Neustadt", DistrictLook::Station),
            ],
            pubs: vec![NamePattern {
                pattern: "Zum {adj} {noun}".into(),
                weight: 1,
            }],
            boss_pubs: vec![NamePattern {
                pattern: "Vereinsheim {noun}".into(),
                weight: 1,
            }],
            spelunken: Vec::new(),
            words: BTreeMap::from([
                (
                    "adj".to_owned(),
                    vec!["Goldenen".to_owned(), "Roten".to_owned()],
                ),
                (
                    "noun".to_owned(),
                    vec!["Hirschen".to_owned(), "Ochsen".to_owned()],
                ),
            ]),
        }
    }

    #[test]
    fn the_generator_fills_patterns_and_is_deterministic() {
        let city = names().generate(42, 3, 5);
        assert_eq!(city, names().generate(42, 3, 5));
        assert_eq!(city.len(), 3);
        let districts: Vec<&String> = city.iter().map(|d| &d.name).collect();
        for (i, d) in districts.iter().enumerate() {
            assert!(!districts[..i].contains(d), "distinct districts");
        }
        for CityDistrict { pubs, .. } in &city {
            assert_eq!(pubs.len(), 5);
            assert!(
                pubs[..4]
                    .iter()
                    .all(|p| p.starts_with("Zum ") && !p.contains('{'))
            );
            assert!(pubs[4].starts_with("Vereinsheim "));
        }
        assert_ne!(
            city,
            names().generate(43, 3, 5),
            "another seed, another city"
        );
    }

    #[test]
    fn every_evening_has_one_district_of_each_look_in_seeded_order() {
        let names = names();
        let mut orders = BTreeSet::new();
        for seed in 0..64 {
            let city = names.generate(seed, 3, 5);
            let mut looks: Vec<DistrictLook> = city.iter().map(|d| d.look).collect();
            orders.insert(looks.clone());
            looks.sort_unstable();
            assert_eq!(
                looks,
                [
                    DistrictLook::OldTown,
                    DistrictLook::Harbour,
                    DistrictLook::Station
                ],
                "seed {seed}: one of each look"
            );
            for d in &city {
                let def = names.districts.iter().find(|n| n.name == d.name);
                assert_eq!(def.map(|n| n.look), Some(d.look), "the name's own look");
            }
        }
        assert_eq!(orders.len(), 6, "every order of the looks occurs");
    }

    #[test]
    fn short_lists_repeat_looks_then_number_districts() {
        let mut names = names();
        names.districts = vec![
            district("Altstadt", DistrictLook::OldTown),
            district("Hafen", DistrictLook::Harbour),
        ];
        let city = names.generate(7, 3, 5);
        let mut found: Vec<&str> = city.iter().map(|d| d.name.as_str()).collect();
        found.sort_unstable();
        assert_eq!(found, ["Altstadt", "Hafen", "Viertel 3"]);
    }
}
