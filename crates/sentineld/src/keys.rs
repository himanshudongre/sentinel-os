use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use ed25519_dalek::SigningKey;
use rand_core::OsRng;
use std::fs;
use std::io;

pub fn load_or_create_signing_key(path: &str) -> io::Result<SigningKey> {
    match fs::read_to_string(path) {
        Ok(s) => {
            let bytes = B64
                .decode(s.trim())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            let arr: [u8; 32] = bytes
                .try_into()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "bad key length"))?;
            Ok(SigningKey::from_bytes(&arr))
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            let sk = SigningKey::generate(&mut OsRng);
            let b64 = B64.encode(sk.to_bytes());
            fs::write(path, format!("{b64}\n"))?;
            Ok(sk)
        }
        Err(e) => Err(e),
    }
}
