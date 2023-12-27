use crate::imports::*;

pub fn sanitize_mnemonic(mnemonic: &str) -> String {
    mnemonic
        .split_ascii_whitespace()
        .filter(|s| s.is_not_empty())
        .collect::<Vec<&str>>()
        .join(" ")
    // let phrase = mnemonic.split_ascii_whitespace().filter(|s| s.is_not_empty()).collect::<Vec<&str>>();
    // phrase.join(" ")
}
