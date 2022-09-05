use crate::env;
use env_logger::{Builder, Env};
use std::io::Write;

pub fn init() {
    if env::var_is_true("CHIM_DEBUG") {
        std::env::set_var("CHIM_LOG_LEVEL", "debug");
    } else if env::var_is_true("CHIM_QUIET") {
        std::env::set_var("CHIM_LOG_LEVEL", "error");
    }
    let env = Env::default().filter("CHIM_LOG_LEVEL");

    Builder::from_env(env)
        .format(|buf, record| {
            let style = buf.default_level_style(record.level());
            writeln!(
                buf,
                "chim[{:5} {}] {}",
                style.value(record.level()),
                record.module_path().unwrap_or(""),
                record.args()
            )
        })
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        init();
    }
}
