// Telemetry decryption module.
// Invariants: decrypted payloads are never logged or persisted in this layer.

use salsa20::cipher::{KeyIvInit, StreamCipher};
use salsa20::Salsa20;

const MAGIC: u32 = 0x47375330;
const KEY_BYTES: &[u8] = b"Simulator Interface Packet GT7 ver 0.0";

pub fn decrypt_packet(dat: &[u8]) -> Option<Vec<u8>> {
    if dat.len() < 0x44 {
        return None;
    }

    let iv1 = u32::from_le_bytes(dat.get(0x40..0x44)?.try_into().ok()?);
    let iv2 = iv1 ^ 0xDEADBEAF;

    let mut nonce = [0u8; 8];
    nonce[0..4].copy_from_slice(&iv2.to_le_bytes());
    nonce[4..8].copy_from_slice(&iv1.to_le_bytes());

    let mut key = [0u8; 32];
    key.copy_from_slice(&KEY_BYTES[0..32]);

    let mut out = dat.to_vec();
    let mut cipher = Salsa20::new(&key.into(), &nonce.into());
    cipher.apply_keystream(&mut out);

    let magic = u32::from_le_bytes(out.get(0..4)?.try_into().ok()?);
    if magic != MAGIC {
        return None;
    }

    Some(out)
}
