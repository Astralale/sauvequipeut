use serde::{Deserialize, Serialize};

// Types pour les messages serveur
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub enum ServerMessage {
    RadarView { RadarView: String },
    Challenge { Challenge: ChallengeType },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ChallengeType {
    SecretSumModulo(u64),
    SOS,
}

pub fn decode_b64(encoded: &str) -> Result<String, String> {
    let b64_alphabet: Vec<char> =
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+/"
            .chars()
            .collect();

    let mut bits = String::new();
    for char in encoded.chars() {
        if let Some(index) = b64_alphabet.iter().position(|&c| c == char) {
            bits.push_str(&format!("{:06b}", index));
        } else {
            return Err(format!(
                "Caractère non valide dans l'encodage b64 : {}",
                char
            ));
        }
    }

    let bytes: Vec<u8> = bits
        .as_bytes()
        .chunks(8)
        .filter_map(|chunk| {
            if chunk.len() == 8 {
                Some(u8::from_str_radix(&String::from_utf8_lossy(chunk), 2).unwrap())
            } else {
                None
            }
        })
        .collect();

    let decoded = bytes
        .iter()
        .enumerate()
        .map(|(i, &byte)| {
            if i < 6 {
                format!("{:08b}", byte)
            } else {
                format!("{:02X}", byte)
            }
        })
        .collect::<Vec<String>>()
        .join(" ");

    Ok(decoded)
}
