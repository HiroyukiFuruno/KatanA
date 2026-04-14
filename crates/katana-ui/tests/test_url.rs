#[test]
fn test_encode() {
    let url_str = "https://img.shields.io/badge/Sponsor-❤️-ea4aaa.svg?style=for-the-badge&logo=github-sponsors";
    let encoded = encode_unicode_uri(url_str);
    println!("Encoded: {}", encoded);
    assert_eq!(
        encoded,
        "https://img.shields.io/badge/Sponsor-%E2%9D%A4%EF%B8%8F-ea4aaa.svg?style=for-the-badge&logo=github-sponsors"
    );
}

fn encode_unicode_uri(url: &str) -> String {
    let mut out = String::with_capacity(url.len());
    for c in url.chars() {
        if c.is_ascii() {
            if c == ' ' {
                out.push_str("%20");
            } else {
                out.push(c);
            }
        } else {
            let mut buf = [0; 4];
            let s = c.encode_utf8(&mut buf);
            for &b in s.as_bytes() {
                use std::fmt::Write;
                let _ = write!(&mut out, "%{:02X}", b);
            }
        }
    }
    out
}
