//! Java-compatible RNG. Everything else in this crate (placement, and
//! transitively conditions/search) depends on this producing bit-for-bit
//! the same output as `java.util.Random` — do not change the constants
//! or the overflow behavior without re-running the known-seeds tests.

// Java's 48-bit linear congruential generator (`java.util.Random`).
// Minecraft uses this for structure placement RNG (region -> chunk offset).
#[derive(Clone, Copy, Debug)]
pub struct JavaLcg {
    seed: i64,
}

impl JavaLcg {
    // Constants are taken from `java.util.Random` source code. See:
    // https://github.com/openjdk/jdk/blob/master/src/java.base/share/classes/java/util/Random.java
    const MULTIPLIER: i64 = 0x5DEECE66D;
    const ADDEND: i64 = 0xB;
    const MASK: i64 = (1 << 48) - 1;

    // The `new` function initializes the RNG with a given seed.
    //It applies the same transformation as Java's `Random` constructor to ensure compatibility.
    #[inline(always)]
    pub fn new(seed: i64) -> Self {
        Self {
            seed: (seed ^ Self::MULTIPLIER) & Self::MASK,
        }
    }

    // The `next_bits` function generates the next random bits from the RNG.
    // It updates the internal seed using the linear congruential formula and returns the requested number of bits.
    #[inline(always)]
    pub fn next_bits(&mut self, bits: u32) -> i32 {
        debug_assert!(bits <= 32, "bits must be <= 32");
        self.seed = self
            .seed
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(Self::ADDEND)
            & Self::MASK;
        (self.seed >> (48 - bits)) as i32
    }

    /// The `next_int` function generates a random integer in the range [0, bound).
    /// Uses a direct bitshift for power-of-two bounds, or rejection sampling otherwise.
    /// Matches the behavior of `java.util.Random.nextInt(bound)`, including Java's 32-bit overflow behavior.
    #[inline(always)]
    pub fn next_int(&mut self, bound: i32) -> i32 {
        debug_assert!(bound > 0, "bound must be positive");

        if (bound & -bound) == bound {
            return ((bound as i64).wrapping_mul(self.next_bits(31) as i64) >> 31) as i32;
        }
        let mut bits = self.next_bits(31);
        let mut val = bits % bound;
        while bits.wrapping_sub(val).wrapping_add(bound - 1) < 0 {
            bits = self.next_bits(31);
            val = bits % bound;
        }
        val
    }
}

/// The tests module contains unit tests to verify the correctness of the JavaLcg implementation.
#[cfg(test)]
mod tests {
    use super::*;

    /// java.util.Random(0).nextInt() == -1155484576. This is `next(32)`,
    /// not `next(31)` — verified independently against the LCG formula
    /// before being pinned here, since a wrong reference value would make
    /// this test lie about correctness instead of catching regressions.
    #[test]
    fn matches_known_java_random_output() {
        let mut rng = JavaLcg::new(0);
        assert_eq!(rng.next_bits(32), -1155484576);
    }

    // Verify that the JavaLcg implementation produces the same output as java.util.Random for specific seeds and bounds.
    #[test]
    fn matches_known_power_of_two_bound_output() {
        let mut rng = JavaLcg::new(0);
        // Exercises the power-of-two fast path, untested by the bound=10 case above
        assert_eq!(rng.next_int(8), 5);
        assert_eq!(rng.next_int(8), 6);
        assert_eq!(rng.next_int(8), 1);
        assert_eq!(rng.next_int(8), 4);
        assert_eq!(rng.next_int(8), 5);
    }

    // Verifies that the JavaLcg implementation produces the same output as java.util.Random for a non-power-of-two bound, exercising the rejection-sampling loop.
    #[test]
    fn matches_known_bounded_output() {
        let mut rng = JavaLcg::new(0);
        // Exercises the rejection-sampling loop (non-power-of-two path)
        assert_eq!(rng.next_int(10), 0);
        assert_eq!(rng.next_int(10), 8);
        assert_eq!(rng.next_int(10), 9);
        assert_eq!(rng.next_int(10), 7);
        assert_eq!(rng.next_int(10), 5);
    }
}
