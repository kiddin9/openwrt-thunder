use super::{CHECK_AUTH, EXP, TOKEN_SECRET};
use anyhow::Result;
use jsonwebtokens::{encode, Algorithm, AlgorithmID, Verifier};
use std::{collections::HashMap, time::Duration};

fn get_or_init_secret() -> &'static String {
    TOKEN_SECRET.get_or_init(|| {
        let secret = if let Some(Some(auth_password)) = CHECK_AUTH.get() {
            auth_password.to_owned()
        } else {
            generate_random_string(31)
        };
        let (x, y) = super::murmur::murmurhash3_x64_128(secret.as_bytes(), 31);
        format!("{x}{y}")
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

pub fn verifier(token_str: &str) -> Result<()> {
    let s = get_or_init_secret();
    let alg = Algorithm::new_hmac(AlgorithmID::HS256, s.to_owned())?;
    let verifier = Verifier::create().build()?;
    let _ = verifier.verify(&token_str, &alg)?;
    Ok(())
}

fn now_duration() -> anyhow::Result<Duration> {
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(std::time::UNIX_EPOCH)?;
    Ok(duration)
}

fn generate_random_string(len: usize) -> String {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let rng = thread_rng();
    rng.sample_iter(&Alphanumeric)
        .take(len)
        .map(|x| CHARSET[x as usize % CHARSET.len()] as char)
        .collect()
}
