//! IfáScript CLI — Agent birth entropy generator
//!
//! Usage:
//!   ifascript cast --intent "birth agent 369"
//!   ifascript cast --intent "birth agent 369" --format json
//!
//! Outputs a JSON object with Odù pattern, entropy seed, and archetype metadata.

use ifascript::odu::{get_odu_by_binary, Odu};
use ifascript::vm::IfaVM;
use sha2::{Digest, Sha256};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let command = args.get(1).map(|s| s.as_str()).unwrap_or("cast");
    let intent = args
        .iter()
        .position(|a| a == "--intent")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("sovereign birth");

    match command {
        "cast" => cast_for_birth(intent),
        "lookup" => {
            if let Some(idx) = args.get(2).and_then(|s| s.parse::<u8>().ok()) {
                lookup_odu(idx);
            } else {
                eprintln!("Usage: ifascript lookup <0-255>");
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!("Commands: cast, lookup");
            std::process::exit(1);
        }
    }
}

fn cast_for_birth(intent: &str) {
    let mut vm = IfaVM::with_intent(intent);

    // Cast 8 cowries to form the 8-bit Odù binary
    let mut odu_bits: [u8; 8] = [0; 8];
    let mut raw_casts: Vec<u16> = Vec::new();

    for i in 0..8 {
        let cast = vm.oracle.cast_cowries();
        raw_casts.push(cast);
        odu_bits[i] = (cast & 1) as u8;
    }

    // Convert 8 bits to u8 binary value for Odù lookup
    let binary_value: u8 = odu_bits
        .iter()
        .enumerate()
        .fold(0u8, |acc, (i, &bit)| acc | (bit << (7 - i)));

    let odu = get_odu_by_binary(binary_value);

    // Generate entropy seed from all cast values + intent
    let mut hasher = Sha256::new();
    hasher.update(intent.as_bytes());
    for cast in &raw_casts {
        hasher.update(cast.to_be_bytes());
    }
    let entropy_hash = hasher.finalize();
    let entropy_hex = hex::encode(&entropy_hash);

    // Generate keypair seed (second hash for domain separation)
    let mut kp_hasher = Sha256::new();
    kp_hasher.update(b"ifascript-keypair-v1:");
    kp_hasher.update(&entropy_hash);
    kp_hasher.update(intent.as_bytes());
    let keypair_seed = hex::encode(kp_hasher.finalize());

    // Output JSON
    println!(
        r#"{{"odu":{{"index":{},"binary":"{}","name":"{}","archetype":"{}","description":"{}","orisha":{:?},"interpretation_type":"{}"}},"entropy":{{"seed":"0x{}","keypair_seed":"0x{}","raw_casts":{:?},"odu_bits":{:?}}},"intent":"{}"}}"#,
        odu.index,
        format!("{:08b}", binary_value),
        odu.name,
        odu.archetype,
        odu.description.replace('"', "'"),
        odu.orisha,
        odu.interpretation_type,
        entropy_hex,
        keypair_seed,
        raw_casts,
        odu_bits,
        intent
    );
}

fn lookup_odu(index: u8) {
    let odu = get_odu_by_binary(index);
    println!(
        r#"{{"index":{},"binary":"{}","name":"{}","archetype":"{}","description":"{}","orisha":{:?},"taboos":{:?},"prescriptions":{:?},"interpretation_type":"{}"}}"#,
        odu.index,
        format!("{:08b}", odu.binary),
        odu.name,
        odu.archetype,
        odu.description.replace('"', "'"),
        odu.orisha,
        odu.taboos,
        odu.prescriptions,
        odu.interpretation_type,
    );
}
