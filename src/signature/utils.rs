use ring::hmac;

pub fn hash(msg: &str) -> String {
    let hash = ring::digest::digest(&ring::digest::SHA256, msg.as_bytes());
    hex::encode(hash.as_ref())
}

pub fn sign(key: &[u8], msg: &str) -> hmac::Tag {
    let k = hmac::Key::new(hmac::HMAC_SHA256, key);
    hmac::sign(&k, msg.as_bytes())
}

pub fn merge(xs: Vec<String>, sep: &str) -> String {
    let mut pairs = xs;
    pairs.sort();
    pairs.join(sep)
}
