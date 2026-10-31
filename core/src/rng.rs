//! Deterministic random number generator for the core.
//!
//! The core must never touch system randomness (see `docs/DESIGN.md`,
//! section 14.2). This is xoshiro256** seeded through splitmix64, which is
//! fast, has a 2^256 period and produces the same sequence on every platform.
//! No external crate is involved so the output can never change under our
//! feet through a dependency update.

use std::ops::Range;

/// A seeded, deterministic PRNG (xoshiro256**).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rng {
    state: [u64; 4],
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

impl Rng {
    /// Create a generator from a 64-bit seed. Equal seeds give equal streams.
    #[must_use]
    pub fn from_seed(seed: u64) -> Rng {
        let mut sm = seed;
        let state = [
            splitmix64(&mut sm),
            splitmix64(&mut sm),
            splitmix64(&mut sm),
            splitmix64(&mut sm),
        ];
        Rng { state }
    }

    /// Next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        let s = &mut self.state;
        let result = s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = s[1] << 17;
        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = s[3].rotate_left(45);
        result
    }

    /// Uniform value in `range` (half open). An empty range yields
    /// `range.start` instead of panicking.
    ///
    /// Uses Lemire's nearly divisionless method, so the result is unbiased.
    pub fn gen_range(&mut self, range: Range<u64>) -> u64 {
        let span = range.end.saturating_sub(range.start);
        if span == 0 {
            return range.start;
        }
        let mut m = u128::from(self.next_u64()) * u128::from(span);
        let mut low = m as u64;
        if low < span {
            let threshold = span.wrapping_neg() % span;
            while low < threshold {
                m = u128::from(self.next_u64()) * u128::from(span);
                low = m as u64;
            }
        }
        range.start + (m >> 64) as u64
    }

    /// In-place Fisher-Yates shuffle.
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.gen_range(0..(i as u64 + 1)) as usize;
            items.swap(i, j);
        }
    }

    /// Derive an independent generator for a sub task (e.g. one game of a
    /// run) so that the parent stream advances by exactly one value.
    pub fn fork(&mut self) -> Rng {
        Rng::from_seed(self.next_u64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Rng::from_seed(42);
        let mut b = Rng::from_seed(42);
        let xs: Vec<u64> = (0..100).map(|_| a.next_u64()).collect();
        let ys: Vec<u64> = (0..100).map(|_| b.next_u64()).collect();
        assert_eq!(xs, ys);
    }

    #[test]
    fn different_seeds_differ() {
        let mut a = Rng::from_seed(1);
        let mut b = Rng::from_seed(2);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn gen_range_stays_in_bounds() {
        let mut rng = Rng::from_seed(7);
        for _ in 0..10_000 {
            let v = rng.gen_range(3..10);
            assert!((3..10).contains(&v));
        }
        assert_eq!(rng.gen_range(5..5), 5);
        assert_eq!(rng.gen_range(9..2), 9);
    }

    #[test]
    fn shuffle_is_permutation_and_deterministic() {
        let mut a: Vec<u32> = (0..32).collect();
        let mut b = a.clone();
        Rng::from_seed(99).shuffle(&mut a);
        Rng::from_seed(99).shuffle(&mut b);
        assert_eq!(a, b);
        let mut sorted = a.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..32).collect::<Vec<_>>());
        assert_ne!(a, sorted, "32 elements should not stay in place");
    }

    #[test]
    fn fork_advances_parent_once() {
        let mut parent = Rng::from_seed(5);
        let mut reference = Rng::from_seed(5);
        let _child = parent.fork();
        reference.next_u64();
        assert_eq!(parent, reference);
    }
}
