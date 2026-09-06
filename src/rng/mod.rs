/* Random number generation utilities.
Uses Java Random and provides a Rust wrapper for it. */

pub struct JavaRandom {
    seed: i64,
}

impl JavaRandom {
    // Constants used in the Java Random algorithm
    const MULTIPLIER: i64 = 0x5DEECE66D;
    const INCREMENT: i64 = 0xB;
    const MASK: i64 = (1 << 48) - 1;

    // Creates a new JavaRandom instance with the given seed.
    pub fn new(seed: i64) -> Self {
        Self {
            seed: (seed ^ Self::MULTIPLIER) & Self::MASK,
        }
    }

    // Generates the next random number with the specified number of bits.
    fn next(&mut self, bits: u32) -> i32 {
        self.seed = self
            .seed
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(Self::INCREMENT)
            & Self::MASK;
        (self.seed >> (48 - bits)) as i32
    }

    // Calls next(32) to generate a random integer.
    pub fn next_int(&mut self) -> i32 {
        self.next(32)
    }

    // Generates a random integer in the range [0, bound).
    pub fn next_int_bound(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "Bound must be positive");

        if (bound & -bound) == bound {
            // bound is a power of two — fast path, no rejection needed
            return ((bound as i64 * self.next(31) as i64) >> 31) as i32;
        }

        loop {
            let bits = self.next(31);
            let val = bits % bound;
            if bits - val + (bound - 1) >= 0 {
                return val;
            }
        }
    }
}

// Unit tests for the JavaRandom implementation.
#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn matches_known_java_output() {
        // Verify that the output matches known Java Random outputs for a given seed
        let mut rng = JavaRandom::new(0);
        assert_eq!(rng.next_int(), -1155484576);
        assert_eq!(rng.next_int(), -723955400);
    }

    #[test]
    fn matches_known_bounded_output() {
        let mut rng = JavaRandom::new(0);
        // Verified sequence for new Random(0).nextInt(10) called 5 times
        assert_eq!(rng.next_int_bound(10), 0);
        assert_eq!(rng.next_int_bound(10), 8);
        assert_eq!(rng.next_int_bound(10), 9);
        assert_eq!(rng.next_int_bound(10), 7);
        assert_eq!(rng.next_int_bound(10), 5);
    }
}
