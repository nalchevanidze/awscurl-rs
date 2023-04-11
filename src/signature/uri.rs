fn escape_char(c: char) -> String {
    let escaped: String = c
        .to_string()
        .bytes()
        .map(|b| format!("{:02X}", b))
        .collect();

    format!("%{}", escaped)
}

fn encode_uri_char(c: char) -> String {
    match c {
        // don't encode unreserved characters
        'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '~' | '.' => c.to_string(),
        // encode reserved characters
        '/' => "%2F".to_string(),
        c => escape_char(c),
    }
}

pub fn encode_uri(uri: &str) -> String {
    uri.chars().map(|c| encode_uri_char(c)).collect()
}
