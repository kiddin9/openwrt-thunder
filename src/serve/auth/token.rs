use jsonwebtokens::{encode, Algorithm, AlgorithmID, Verifier};
use std::{collections::HashMap, sync::OnceLock, time::Duration};

use super::CHECK_AUTH;

static TOKEN_SECRET: OnceLock<String> = OnceLock::new();
pub(super) const EXP: u64 = 3600 * 24;

fn get_or_init_secret() -> &'static String {
    TOKEN_SECRET.get_or_init(|| {
        let s = if let Some(Some(auth_password)) = CHECK_AUTH.get() {
            let (x, y) = super::murmur::murmurhash3_x64_128(auth_password.as_bytes(), 31);
            format!("{x}{y}")
        } else {
            let (x, y) = super::murmur::murmurhash3_x64_128(b"fuck", 31);
            format!("{x}{y}")
        };
        s
    })
}

pub fn generate_token() -> anyhow::Result<String> {
    let s = get_or_init_secret();
    let alg = Algorithm::new_hmac(AlgorithmID::HS256, s.to_owned())?;

    let mut header = HashMap::new();
    let mut claims = HashMap::new();
    header.insert("alg".to_owned(), alg.name().to_owned());
    claims.insert("exp".to_owned(), now_duration()?.as_secs() + EXP);

    Ok(encode(&header, &claims, &alg)?)
}

pub fn verifier(token_str: &str) -> anyhow::Result<()> {
    let s = get_or_init_secret();
    let alg = Algorithm::new_hmac(AlgorithmID::HS256, s.to_owned())?;
    let verifier = Verifier::create().build()?;
    let _ = verifier.verify(&token_str, &alg)?;
    Ok(())
}

pub fn now_duration() -> anyhow::Result<Duration> {
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(std::time::UNIX_EPOCH)?;
    Ok(duration)
}
