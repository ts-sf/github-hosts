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
            "vv.cv",
            "dddddddddddd.dddddddd.d.d.d.d.d.d",
            "4.cbn",
            "c-56.10321.com",
            "u8am-9d5.l-----asd.3vv",
        ];

        for d in domains {
            assert!(is_valid_domain(&d), "{}", d);
        }
    }
}
