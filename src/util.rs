use regex::Regex;

const EXPECT_DOMAIN: &str =
    r#"^([a-zA-Z0-9][-a-zA-Z0-9]{0,62})(\.([a-zA-Z0-9][-a-zA-Z0-9]{0,62}))+$"#;

pub fn is_valid_domain(v: &str) -> bool {
    let re = Regex::new(EXPECT_DOMAIN).unwrap();
    re.is_match(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_domain() {
        let domains = [
            "vv.cv".to_owned(),
            "dddddddddddd.dddddddd.d.d.d.d.d.d".to_owned(),
            "4.cbn".to_owned(),
            "c-56.10321.com".to_owned(),
            "u8am-9d5.l-----asd.3vv".to_owned(),
        ];

        for d in domains {
            assert!(is_valid_domain(&d), "{}", d);
        }
    }
}
