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
