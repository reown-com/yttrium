pub const GIT_HASH: &str = env!("GIT_HASH");

pub fn format_sdk_version() -> String {
    format!("yttrium-{GIT_HASH}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_sdk_version() {
        assert_eq!(format_sdk_version(), format!("yttrium-{GIT_HASH}"));
    }

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_git_hash() {
        assert_ne!(GIT_HASH, "unknown");
        assert!(!GIT_HASH.is_empty());
        assert_eq!(GIT_HASH.len(), 40);
    }
}
