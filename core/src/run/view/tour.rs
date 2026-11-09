//! The view of the Kneipentour ([`Run::tour_view`]): the city map with
//! every pub, the route choice and the trip under way. The data types
//! stay in [`super::super::tour`], where frontends find them.

use super::super::guests::seat_guests;
use super::super::tour::{
    DistrictView, MalusInfo, PubNode, PubPreview, PubState, RouteOptionView, RouteView, Spelunke,
    SpelunkeNode, StopEncounter, StopView, TourView, TripView, pub_id,
};
use super::super::{PUBS_PER_DISTRICT, Run, RunPhase};
use crate::content::{CityDistrict, Content, MalusDef};

impl Run {
    /// Preview of a pub: name, target, who sits at the table.
    fn pub_preview(
        &self,
        content: &Content,
        city: &[CityDistrict],
        section: u32,
        index: u32,
        spelunke: Option<&Spelunke>,
    ) -> Option<PubPreview> {
        let is_boss = index + 1 == PUBS_PER_DISTRICT;
        // A new district's boss is drawn when it begins.
        let boss = if is_boss && section == self.series.section {
            self.section_boss.as_ref()
        } else {
            None
        };
        let mut target = self.pub_target(content, section, index, boss)?;
        if spelunke.is_some() {
            target = content.run.spelunke.target_factor.apply(target);
        }
        if (section, index) != (self.series.section, self.series.index) {
            target = self.next_pub.apply_to(target);
        }
        let district = city.get(usize::try_from(section).ok()?)?;
        let name = spelunke.map_or_else(
            || district.pubs.get(usize::try_from(index).ok()?).cloned(),
            |s| Some(s.name.clone()),
        )?;
        let guests = seat_guests(content, self.seed, section, index, &self.guests)
            .iter()
            .zip(crate::game::Seat::DEFENDERS)
            .filter_map(|(id, seat)| self.guest_info(content, id, seat))
            .collect();
        Some(PubPreview {
            id: pub_id(section, index, spelunke.is_some()),
            section,
            index,
            name,
            district: district.name.clone(),
            target,
            game_limit: content.run.game_limit,
            is_boss,
            boss: boss.map(|id| self.boss_info(content, id)),
            guests,
            malus: spelunke
                .and_then(|s| content.maluses.get(&s.malus))
                .map(malus_info),
        })
    }

    /// The Kneipentour for a frontend.
    #[must_use]
    pub fn tour_view(&self, content: &Content) -> TourView {
        let city = self.city(content);
        let current = (self.series.section, self.series.index);
        let route = self.route.as_ref().and_then(|r| {
            let next = self.pub_preview(content, &city, r.section, r.index, None)?;
            let options = r
                .options
                .iter()
                .enumerate()
                .map(|(i, o)| RouteOptionView {
                    id: format!("{}-r{i}", pub_id(r.section, r.index, false)),
                    index: i,
                    transport: o.transport,
                    minutes: self.minutes(r.section, r.index, o.transport),
                    can_choose: self.accepts(content, super::RunAction::ChooseRoute(i)),
                    stops: o
                        .stops
                        .iter()
                        .map(|stop| StopView {
                            category: stop.category,
                            encounter: (self.scout_legs > 0)
                                .then_some(stop.encounter.as_ref())
                                .flatten()
                                .and_then(|id| content.encounter(id))
                                .map(|e| StopEncounter {
                                    id: e.id.clone(),
                                    name: e.name.clone(),
                                }),
                        })
                        .collect(),
                    malus_options: o
                        .spelunke
                        .as_ref()
                        .map(|s| {
                            s.malus_options
                                .iter()
                                .filter_map(|m| content.maluses.get(m))
                                .map(malus_info)
                                .collect()
                        })
                        .unwrap_or_default(),
                    spelunke: o.spelunke.as_ref().and_then(|s| {
                        self.pub_preview(content, &city, r.section, r.index, Some(s))
                    }),
                })
                .collect();
            Some(RouteView { next, options })
        });
        let done = |section: u32, index: u32| {
            self.pub_results
                .iter()
                .rev()
                .find(|p| p.section == section && p.index == index)
        };
        let districts = city
            .iter()
            .enumerate()
            .map(|(d, district)| {
                let section = u32::try_from(d).unwrap_or(0);
                let mut nodes = Vec::new();
                let mut spelunken = Vec::new();
                for (p, pub_name) in district.pubs.iter().enumerate() {
                    let index = u32::try_from(p).unwrap_or(0);
                    let here = (section, index) == current;
                    let result = done(section, index);
                    let playing = here && matches!(self.phase, RunPhase::Game | RunPhase::Lost);
                    let state = if playing {
                        PubState::Current
                    } else if let Some(r) = result {
                        if r.won { PubState::Won } else { PubState::Lost }
                    } else {
                        PubState::Ahead
                    };
                    let spelunke = if here && self.series.spelunke.is_some() {
                        self.series.spelunke.as_ref().map(|s| s.name.clone())
                    } else {
                        result.and_then(|r| r.spelunke.clone())
                    };
                    if let Some(spelunke) = spelunke {
                        // The Spelunke is a place of its own; the pub it
                        // stood in for was passed by.
                        spelunken.push(SpelunkeNode {
                            id: pub_id(section, index, true),
                            index,
                            name: spelunke,
                            state,
                        });
                        nodes.push(PubNode {
                            id: pub_id(section, index, false),
                            index,
                            name: pub_name.clone(),
                            is_boss: false,
                            state: PubState::Skipped,
                        });
                    } else {
                        nodes.push(PubNode {
                            id: pub_id(section, index, false),
                            index,
                            name: pub_name.clone(),
                            is_boss: index + 1 == PUBS_PER_DISTRICT,
                            state,
                        });
                    }
                }
                DistrictView {
                    index: section,
                    name: district.name.clone(),
                    look: district.look,
                    boss: (section == self.series.section)
                        .then_some(self.section_boss.as_ref())
                        .flatten()
                        .map(|id| self.boss_info(content, id)),
                    map_seed: self.map_seed(Some(section)),
                    pubs: nodes,
                    spelunken,
                }
            })
            .collect();
        let current_pub = if matches!(self.phase, RunPhase::Won) {
            None
        } else {
            self.pub_preview(
                content,
                &city,
                self.series.section,
                self.series.index,
                self.series.spelunke.as_ref(),
            )
            .map(|mut p| {
                p.target = self.series.target;
                if p.malus.is_none() {
                    p.malus = self.malus(content).map(malus_info);
                }
                p
            })
        };
        TourView {
            map_seed: self.map_seed(None),
            districts,
            current: current_pub,
            route,
            trip: self.trip.as_ref().map(|t| TripView {
                transport: t.transport,
                stops_left: t.stops.len(),
                spelunke: t.spelunke.is_some(),
            }),
        }
    }
}

fn malus_info(def: &MalusDef) -> MalusInfo {
    MalusInfo {
        id: def.id.clone(),
        name: def.name.clone(),
        text: def.text.clone(),
    }
}
