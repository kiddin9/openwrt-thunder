use rsa::pkcs1::EncodeRsaPublicKey;

fn main() {
    use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};

    let mut rng = rand::thread_rng();
    let bits = 2048;
    let priv_key: RsaPrivateKey =
        RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let pub_key = RsaPublicKey::from(&priv_key);
    let p = pub_key.to_pkcs1_pem(rsa::pkcs8::LineEnding::LF).unwrap();
    println!("{}", p);
    // Encrypt
    let data = b"hello world";
    let enc_data = pub_key
        .encrypt(&mut rng, Pkcs1v15Encrypt, &data[..])
        .expect("failed to encrypt");
    assert_ne!(&data[..], &enc_data[..]);

    // Decrypt
    let dec_data = priv_key
        .decrypt(Pkcs1v15Encrypt, &enc_data)
        .expect("failed to decrypt");
    assert_eq!(&data[..], &dec_data[..]);
    println!("{}", String::from_utf8_lossy(dec_data.as_ref()))
}
