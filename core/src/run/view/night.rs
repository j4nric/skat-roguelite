//! The view of the long night ([`Run::night_view`], ADR 0020). The data
//! types stay in [`super::super::night`].

use super::super::night::{NightView, VerschaerfungInfo};
use super::super::{Run, RunPhase};
use crate::content::{Content, NightRule};

impl Run {
    /// The long night for a frontend; `None` outside it.
    #[must_use]
    pub fn night_view(&self, content: &Content) -> Option<NightView> {
        let night = self.night.as_ref()?;
        let sections = content.run.sections();
        let ahead = self
            .route
            .as_ref()
            .map(|r| (r.section, r.index))
            .or(self.trip.as_ref().map(|t| (t.section, t.index)));
        let section = match (ahead, self.phase) {
            (Some((s, _)), _) => s,
            (None, RunPhase::Won) => sections,
            _ => self.series.section,
        };
        let verschaerfungen: Vec<VerschaerfungInfo> = night
            .verschaerfungen
            .iter()
            .enumerate()
            .filter_map(|(i, id)| {
                let def = content.verschaerfungen.get(id)?;
                Some(VerschaerfungInfo {
                    id: id.clone(),
                    name: def.name.clone(),
                    text: def.text.clone(),
                    district: sections + u32::try_from(i).unwrap_or(0) + 1,
                })
            })
            .collect();
        let announced = match ahead {
            Some((s, 0)) if s >= sections => verschaerfungen
                .iter()
                .find(|v| v.district == s + 1)
                .cloned(),
            _ => None,
        };
        let extra_bosses = if section == self.series.section {
            night
                .extra_bosses
                .iter()
                .map(|id| self.boss_info(content, id))
                .collect()
        } else {
            Vec::new()
        };
        let probe = 1_000_000;
        let base = self
            .night_config(content)
            .grow(probe, section.saturating_sub(sections.saturating_sub(1)));
        let mut grown = base;
        for rule in self.night_rules(content) {
            if let NightRule::TargetJump(factor) = rule {
                grown = factor.scale(grown);
            }
        }
        Some(NightView {
            district: section + 1,
            verschaerfungen,
            announced,
            extra_bosses,
            pubs_won: night.pubs_won,
            target_permille: grown / 1000,
        })
    }
}
