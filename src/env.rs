use std::env;

pub fn var_is_true(key: &str) -> bool {
    match env::var(key) {
        Ok(v) => v == "true" || v == "1",
        Err(_) => false,
    }
}

pub fn var_is_false(key: &str) -> bool {
    match env::var(key) {
        Ok(v) => v == "false" || v == "0",
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::env;
    use test_case::test_case;

    #[test_case(Some("true"), true)]
    #[test_case(Some("1"), true)]
    #[test_case(Some("something"), false)]
    #[test_case(Some("0"), false)]
    #[test_case(Some("false"), false)]
    #[test_case(None, false)]
    fn test_var_is_true(value: Option<&str>, expected: bool) {
        let key = get_key();
        match value {
            Some(v) => env::set_var(&key, v),
            None => env::remove_var(&key),
        };
        assert_eq!(var_is_true(&key), expected);
        env::remove_var(&key);
    }

    #[test_case(Some("true"), false)]
    #[test_case(Some("1"), false)]
    #[test_case(Some("something"), false)]
    #[test_case(Some("0"), true)]
    #[test_case(Some("false"), true)]
    #[test_case(None, false)]
    fn test_var_is_false(value: Option<&str>, expected: bool) {
        let key = get_key();
        match value {
            Some(v) => env::set_var(&key, v),
            None => env::remove_var(&key),
        };
        assert_eq!(var_is_false(&key), expected);
        env::remove_var(&key);
    }

    // #[quickcheck]
    fn get_key() -> String {
        format!("CHIM_TEST_KEY_{}", rand::random::<u32>())
    }
}
