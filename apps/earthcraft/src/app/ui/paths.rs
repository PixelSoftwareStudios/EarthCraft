//! Compact path labels shared by file statuses and sidebar folders.
/// Retain recognizable trailing components and hide machine-specific parents.
pub(super) fn short_folder(path: &str) -> String {
    let home = std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" });
    compact_path(
        path,
        home.as_deref().and_then(|home| home.to_str()),
        cfg!(windows),
    )
}
fn compact_path(path: &str, home: Option<&str>, windows: bool) -> String {
    if let Some(home) = home.filter(|home| !home.is_empty()) {
        let normalized_path = if windows {
            path.replace('\\', "/")
        } else {
            path.to_string()
        };
        let normalized_home = if windows {
            home.replace('\\', "/")
        } else {
            home.to_string()
        };
        let home = normalized_home.trim_end_matches('/');
        let prefix = normalized_path.get(..home.len());
        if prefix.is_some_and(|prefix| {
            if windows {
                prefix.eq_ignore_ascii_case(home)
            } else {
                prefix == home
            }
        }) {
            let rest = &normalized_path[home.len()..];
            if rest.is_empty() || rest.starts_with('/') {
                return if windows {
                    format!("%USERPROFILE%{}", rest.replace('/', "\\"))
                } else {
                    format!("~{rest}")
                };
            }
        }
    }
    const VISIBLE_PATH_COMPONENTS: usize = 2;
    // Accept both separators so imported Windows paths also display sensibly.
    let components: Vec<_> = path
        .split(['/', '\\'])
        .filter(|part| !part.is_empty())
        .collect();
    if components.len() <= VISIBLE_PATH_COMPONENTS {
        return path.to_string();
    }
    format!(
        ".../{}",
        components[components.len() - VISIBLE_PATH_COMPONENTS..].join("/")
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn home_abbreviation_requires_a_complete_directory_prefix() {
        assert_eq!(
            compact_path("/Users/ana/Pictures/EarthCraft", Some("/Users/ana"), false),
            "~/Pictures/EarthCraft"
        );
        assert_eq!(compact_path("/Users/ana", Some("/Users/ana"), false), "~");
        assert_eq!(
            compact_path(
                "/Users/anabel/Pictures/EarthCraft",
                Some("/Users/ana"),
                false
            ),
            ".../Pictures/EarthCraft"
        );
        assert_eq!(
            compact_path(
                r"C:\Users\Ana\Pictures\EarthCraft",
                Some(r"c:\users\ana"),
                true
            ),
            r"%USERPROFILE%\Pictures\EarthCraft"
        );
    }
    #[test]
    fn preserves_filename_and_unicode_without_expanding_short_paths() {
        assert_eq!(
            compact_path("/Users/name/Downloads/São Paulo.kml", None, false),
            ".../Downloads/São Paulo.kml"
        );
        assert_eq!(
            compact_path(r"C:\Users\name\Downloads\places.kml", None, true),
            ".../Downloads/places.kml"
        );
        assert_eq!(short_folder("places.kml"), "places.kml");
        assert_eq!(short_folder(""), "");
    }
}
