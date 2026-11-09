//! Small helpers of the run without state of their own.

use crate::content::{CardModDef, CardModId, Content};
use crate::rng::Rng;

/// The value a modification goes on with: the one named, else the first on
/// offer only if it is the named modification (a random draw takes its
/// own first value).
pub(super) fn mod_value(content: &Content, card_mod: &CardModId, named: Option<i32>) -> i32 {
    named.unwrap_or_else(|| {
        content
            .card_mod(card_mod)
            .map_or(1, CardModDef::default_value)
    })
}

/// Draw up to `count` distinct items by their weight (items of weight 0
/// never), in a fixed pool order.
pub(super) fn draw_weighted<T: Clone>(
    rng: &mut Rng,
    mut pool: Vec<(T, u32)>,
    count: u32,
) -> Vec<T> {
    let mut drawn = Vec::new();
    for _ in 0..count {
        let total: u64 = pool.iter().map(|(_, w)| u64::from(*w)).sum();
        if total == 0 {
            break;
        }
        let mut roll = rng.gen_range(0..total);
        let mut chosen = None;
        for (i, (_, weight)) in pool.iter().enumerate() {
            let w = u64::from(*weight);
            if roll < w {
                chosen = Some(i);
                break;
            }
            roll -= w;
        }
        let Some(i) = chosen else {
            break;
        };
        drawn.push(pool.remove(i).0);
    }
    drawn
}
