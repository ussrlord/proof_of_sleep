use blake2b_simd::blake2b;
use parking_lot::RwLock;
use rand::{thread_rng, Rng};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Clone)]
pub struct POS {
    target: usize,
    delay: u64,
    is_interrupt: Arc<RwLock<bool>>,
}

impl POS {
    pub fn new(target: usize, delay: u64) -> POS {
        POS {
            target,
            delay,
            is_interrupt: Arc::new(RwLock::new(false)),
        }
    }

    pub fn mint(&self, block_hash: &[u8]) -> Option<u64> {
        loop {
            if *self.is_interrupt.read() {
                return None;
            }
            let nonce: u64 = thread_rng().gen();
            if self.check_nonce(block_hash, nonce) {
                return Some(nonce);
            }
            thread::sleep(Duration::from_secs(self.delay));
        }
    }

    pub fn check_nonce(&self, block_hash: &[u8], nonce: u64) -> bool {
        let nonce_bytes = nonce.to_be_bytes();
        let mut bytes = nonce_bytes[0..].to_vec();
        bytes.extend(block_hash);
        let hash = blake2b(&bytes);
        let mut target = self.target;
        for v in hash.as_bytes() {
            if v == 0 {
                if target <= 8 {
                    return true;
                }
                target -= 8;
            } else {
                let lz = u8::leading_zeros(*v);
                if lz <= target {
                    return false;
                } else {
                    return true;
                }
            }
        }
        false
    }

    pub fn interrupt(&self) {
        let mut is_interrupt = self.is_interrupt.write();
        *is_interrupt = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mint() {
        let p = POS::new(2, 0);
        let block_hash = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let nonce = p.mint(&block_hash).unwrap();
        assert!(p.check_nonce(&block_hash, nonce));
    }

    #[test]
    fn interrupt() {
        let p = POS::new(2, 1);
        let block_hash = vec![1, 2, 3, 4, 5, 6, 7, 8];
        p.interrupt();
        assert_eq!(p.mint(&block_hash), None);
    }
}
