use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand_core::OsRng;
use sha2::{Digest, Sha256};

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

pub fn generate_signing_key() -> SigningKey {
    SigningKey::generate(&mut OsRng)
}

pub fn sign_bytes(sk: &SigningKey, msg: &[u8]) -> Signature {
    sk.sign(msg)
}

pub fn verify_bytes(
    vk: &VerifyingKey,
    msg: &[u8],
    sig: &Signature,
) -> Result<(), ed25519_dalek::SignatureError> {
    vk.verify(msg, sig)
}

pub fn pubkey_id_from_vk(vk: &ed25519_dalek::VerifyingKey) -> String {
    sha256_hex(vk.to_bytes().as_slice())
}
