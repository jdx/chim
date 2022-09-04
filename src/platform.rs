use std::borrow::Cow;

pub fn split_platform_name(name: &str) -> (Cow<'static, str>, Cow<'static, str>) {
    let mut parts = name.split('-');
    return (
        normalize_os(parts.next().unwrap()),
        normalize_arch(parts.next().unwrap()),
    );
}

fn normalize_os(os: &str) -> Cow<'static, str> {
    match os {
        "darwin" => "macos".into(),
        "win" => "windows".into(),
        _ => os.to_owned().into(),
    }
}

fn normalize_arch(arch: &str) -> Cow<'static, str> {
    match arch {
        "arm64" => "aarch64".into(),
        "x64" => "x86_64".into(),
        _ => arch.to_owned().into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("darwin-x64", "macos-x86_64")]
    #[test_case("macos-x64", "macos-x86_64")]
    #[test_case("macos-arm64", "macos-aarch64")]
    #[test_case("linux-aarch64", "linux-aarch64")]
    fn test_split_platform_name(input: &str, expected: &str) {
        let (os, arch) = split_platform_name(input);
        assert_eq!(format!("{os}-{arch}"), expected);
    }

    #[test]
    fn test_normalize_os() {
        assert_eq!(normalize_os("darwin"), "macos");
        assert_eq!(normalize_os("win"), "windows");
        assert_eq!(normalize_os("linux"), "linux");
    }

    #[test]
    fn test_normalize_arch() {
        assert_eq!(normalize_arch("arm64"), "aarch64");
        assert_eq!(normalize_arch("x64"), "x86_64");
        assert_eq!(normalize_arch("x86"), "x86");
    }
}
