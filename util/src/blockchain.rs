use ed25519_dalek::{ed25519::signature::SignerMut, Signature, SigningKey};
pub fn address_brief(address: &String) -> String {
    let mut result = String::new();
    let len = address.len();
    if len <= 10 {
        result.push_str(&address);
    } else {
        result.push_str(&address[..4]);
        result.push_str("...");
        result.push_str(&address[len - 4..]);
    }
    result
}
pub fn sha256(message: String) -> anyhow::Result<String> {
    let message_bytes = message.as_bytes();
    // sha256 hash the message
    let mut hasher = Sha256::new();

    // Write input message
    hasher.update(&message_bytes);

    // Read hash digest and consume hasher
    let result = hasher.finalize();
    Ok(hex::encode(result))
}
pub fn address_from_keypair_25519(keypair_hex: String) -> anyhow::Result<String> {
    let signing_key = signing_key_from_bytes_25519(keypair_hex)?;
    let mut public_key: [u8; 20] = [0; 20];
    public_key.copy_from_slice(&signing_key.verifying_key().as_bytes()[0..20]);
    let mut hasher = Sha256::new();
    hasher.update(&signing_key.verifying_key().as_bytes());
    let pub_hash = &hasher.finalize()[..20];
    Ok(hex::encode(pub_hash))
}
pub fn signing_key_from_bytes_25519(keypair_hex: String) -> anyhow::Result<SigningKey> {
    let private_key_bytes = hex::decode(keypair_hex)?;
    let keypair_bytes = private_key_bytes.as_slice().try_into()?;
    let signing_key = ed25519_dalek::SigningKey::from_keypair_bytes(keypair_bytes)?;
    Ok(signing_key)
}
pub fn sign_message_25519(keypair_hex: String, message: String) -> anyhow::Result<String> {
    let mut signing_key = signing_key_from_bytes_25519(keypair_hex.clone())?;
    let hash = sha256(message.clone())?;
    let message_bytes = hex::decode(hash)?;

    let signature = signing_key.sign(&message_bytes);
    assert!(verify_message_25519(
        keypair_hex,
        message.clone(),
        hex::encode(signature.to_bytes()),
    )
    .is_ok());
    Ok(hex::encode(signature.to_bytes()))
}
pub fn verify_message_25519(
    keypair_hex: String,
    message: String,
    signature_hex: String,
) -> anyhow::Result<()> {
    let signing_key = signing_key_from_bytes_25519(keypair_hex)?;
    let signature_bytes = hex::decode(signature_hex)?;
    let signature = Signature::from_bytes(signature_bytes.as_slice().try_into()?);
    let hash = sha256(message)?;
    let message_bytes = hex::decode(hash)?;
    Ok(signing_key.verify(&message_bytes, &signature)?)
}
use anyhow::Ok;
use sha2::{Digest, Sha256};
#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_keypair() {
        let private_key_hex_str =
            "44da02ea3d3829415ff1175467c5f1cf9e3b4b90ef740758e2d9bccbb2520b1971492d9da0d7c2f82bc28b18ee17a34a58656963e022cf1d43143ca788f81510";
        let signing_key = signing_key_from_bytes_25519(private_key_hex_str.to_string()).unwrap();
        assert_eq!(
            hex::encode(&signing_key.to_bytes()),
            "44da02ea3d3829415ff1175467c5f1cf9e3b4b90ef740758e2d9bccbb2520b19"
        );
        assert_eq!(
            hex::encode(&signing_key.verifying_key().to_bytes()),
            "71492d9da0d7c2f82bc28b18ee17a34a58656963e022cf1d43143ca788f81510"
        );
        // let binding =
        //     hex::decode("621d60680125f163026703937914fb092f5ffbabf8f403d39bf711693530a67a")
        //         .unwrap();
        let message = "needsignmessage".to_string();
        let sig = sign_message_25519(private_key_hex_str.to_string(), message.clone()).unwrap();
        assert_eq!(
            sig,
            "ff51d095511c82d28ffb7bed9f65cefa0e7e486b22a5c7b5afb1a1ec6e79098efe21c0a732f2214d53cc0e0cd2e6d1c907863eb21c57ab37081dbd969301c409"
        );
        assert!(verify_message_25519(private_key_hex_str.to_string(), message, sig).is_ok());
        assert_eq!(
            address_from_keypair_25519(private_key_hex_str.to_string()).unwrap(),
            "d2743571aeb3cea7059f08de20d9a3a4a44f85e9".to_string()
        );
    }
}
