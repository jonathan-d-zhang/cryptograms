//! This file contains a Mock RngCore impl to test ciphers with a random
//! component

use rand_core::{impls, RngCore};
pub struct MockRng;

impl RngCore for MockRng {
    fn next_u32(&mut self) -> u32 {
        0
    }
    fn next_u64(&mut self) -> u64 {
        impls::next_u64_via_u32(self)
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

        input.shuffle(&mut MockRng);

        assert_eq!(input, expected);
    }

    #[test]
    fn test_next_u32() {
        assert_eq!(MockRng.next_u32(), 0)
    }

    #[test]
    fn test_next_u64() {
        assert_eq!(MockRng.next_u64(), 0)
    }

    #[test]
    fn test_fill_bytes() {
        let mut arr = vec![0; 5];
        MockRng.fill_bytes(&mut arr);

        assert!(arr == vec![0; 5]);
    }
}
