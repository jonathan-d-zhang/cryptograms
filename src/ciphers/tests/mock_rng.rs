//! This file contains a Mock RngCore impl to test ciphers with a random
//! component

use rand_core::{impls, RngCore};
pub struct MockRng {
    pub state: u64,
}

impl MockRng {
    pub fn new() -> Self {
        Self {
            state: 0,
        }
    }
}

impl RngCore for MockRng {
    fn next_u32(&mut self) -> u32 {
        let r = self.state as u32;
        self.state += 1;

        r
    }

    fn next_u64(&mut self) -> u64 {
        let r = self.state;
        self.state += 1;

        r
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        impls::fill_bytes_via_next(self, dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;

    #[test]
    fn test_shuffle() {
        let mut input = vec![1, 2, 3, 4, 5];
        let expected = vec![2, 3, 4, 5, 1];

        input.shuffle(&mut MockRng::new());

        assert_eq!(input, expected);
    }

    #[test]
    fn test_next_u32() {
        let mut rng = MockRng::new();
        for i in 0..10 {
            assert_eq!(rng.next_u32(), i)
        }
    }

    #[test]
    fn test_next_u64() {
        let mut rng = MockRng::new();
        for i in 0..10 {
            assert_eq!(rng.next_u64(), i)
        }
    }

    #[test]
    fn test_fill_bytes() {
        let mut arr = vec![0; 5];
        MockRng::new().fill_bytes(&mut arr);

        assert!(arr == vec![0; 5]);
    }
}
