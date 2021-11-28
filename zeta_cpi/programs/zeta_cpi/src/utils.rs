use sha2::{Digest, Sha256};

pub fn sighash(namespace: &str, name: &str) -> [u8; 8] {
  let preimage = format!("{}:{}", namespace, name);

  let mut hasher = Sha256::new();
  hasher.update(preimage.as_bytes());
  let result = hasher.finalize();

  let mut sighash = [0u8; 8];
  sighash.copy_from_slice(&result[..8]);
  sighash
}