use std::fmt::Display;
use std::str::pattern::Pattern;

/// A compiled pattern. Like `glob` pattern, but only '*' wildcard is supported.
/// `*` matches any (possibly empty) sequence of characters.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GlobStarPattern {
    /// Blocks of text to be matched literally
    /// The represented pattern is equivalent to `literal_blocks.join("*")`
    literal_blocks: Vec<String>,
}

impl GlobStarPattern {
    /// Returns the number of '*'-wildcards in the pattern
    ///
    /// # Examples
    /// ```
    /// use mmv_lib::GlobStarPattern;
    /// let compiled_pattern = GlobStarPattern::from("IMG*:*:*2010.png");
    /// assert_eq!(compiled_pattern.wildcards_number(), 3);
    /// ```
    pub fn wildcards_number(&self) -> usize {
        self.literal_blocks.len() - 1
    }

    /// Matches the `string` to the pattern.
    /// It the string matches, returns `Vec<&str>` of `string`'s substrings matched by wildcards.
    /// Otherwise returns `None`.
    ///
    /// # Examples
    /// ```
    /// use mmv_lib::GlobStarPattern;
    /// let png_pattern = GlobStarPattern::from("*.png");
    /// assert_eq!(
    ///     png_pattern.match_string("ferris.png"),
    ///     vec!["ferris"].into(),
    /// );
    /// assert!(png_pattern.match_string("ferris.jpg").is_none());
    ///
    /// let rust_lang_pattern = GlobStarPattern::from("rust_*_language.*");
    /// assert_eq!(
    ///     rust_lang_pattern.match_string("rust_is_the_best_language.rs"),
    ///     vec!["is_the_best", "rs"].into(),
    /// );
    /// ```
    pub fn match_string<'a>(&self, mut string: &'a str) -> Option<Vec<&'a str>> {
        string = self.literal_blocks[0].strip_prefix_of(string)?;
        if self.wildcards_number() == 0 {
            return if string.is_empty() {
                Some(vec![])
            } else {
                None
            };
        }
        let mut result = Vec::with_capacity(self.wildcards_number());
        for block in self
            .literal_blocks
            .iter()
            .skip(1)
            .take(self.wildcards_number() - 1)
        {
            let block_match_begin = string.find(block)?;
            result.push(&string[..block_match_begin]);
            string = &string[block_match_begin + block.len()..];
        }
        result.push(
            self.literal_blocks
                .last()
                .unwrap()
                .strip_suffix_of(string)?,
        );
        debug_assert_eq!(result.len(), self.wildcards_number());
        Some(result)
    }
}

#[cfg(test)]
mod test_match_string {
    use super::GlobStarPattern;

    fn check(pattern: &str, test_data: &[(&str, Option<Vec<&str>>)]) {
        let compiled_pattern = GlobStarPattern::from(pattern);
        for (string_to_match, expected_result) in test_data {
            if let Some(match_result) = expected_result {
                assert_eq!(match_result.len(), compiled_pattern.wildcards_number());
            }
            assert_eq!(
                compiled_pattern.match_string(string_to_match),
                *expected_result
            );
        }
    }

    #[test]
    fn single_star() {
        check(
            "*",
            &[
                ("hello", Some(vec!["hello"])),
                ("", Some(vec![""])),
                ("e", Some(vec!["e"])),
            ],
        );
    }

    #[test]
    fn no_wildcard() {
        check(
            "pattern",
            &[
                ("pattern", Some(vec![])),
                ("42", None),
                ("", None),
                ("patternpattern", None),
                ("junkpattern", None),
                ("patternjunk", None),
            ],
        );
    }

    #[test]
    fn simple() {
        check(
            "*.jpg",
            &[
                ("filename.jpg", Some(vec!["filename"])),
                (".jpg", Some(vec![""])),
                ("best_project.rs", None),
                ("", None),
                ("image.jpg.zip", None),
            ],
        );
    }

    #[test]
    fn star_dot_star() {
        check(
            "*.*",
            &[
                ("42.rs", Some(vec!["42", "rs"])),
                (".", Some(vec!["", ""])),
                (".rs", Some(vec!["", "rs"])),
                ("42.", Some(vec!["42", ""])),
                ("nodotrs", None),
                ("", None),
            ],
        );
    }

    #[test]
    fn two_wildcards() {
        check(
            "file*.*",
            &[
                ("file.jpg", Some(vec!["", "jpg"])),
                ("file.", Some(vec!["", ""])),
                ("filename.png", Some(vec!["name", "png"])),
                ("filenoext", None),
                ("file", None),
            ],
        );
    }

    #[test]
    fn repeated_pattern() {
        check(
            "*.rs*.rs",
            &[
                ("file.rs", None),
                ("", None),
                (".rs.rs", Some(vec!["", ""])),
                ("file.rs.rs", Some(vec!["file", ""])),
                ("file.rs42.rs", Some(vec!["file", "42"])),
                (".rs.rsjunk", None),
            ],
        )
    }
}

impl From<&str> for GlobStarPattern {
    fn from(string: &str) -> Self {
        Self {
            literal_blocks: string.split('*').map(str::to_string).collect(),
        }
    }
}

#[test]
fn test_parse_from_str() {
    fn check(pattern: &str, expected_blocks: &[&str]) {
        assert_eq!(
            GlobStarPattern::from(pattern).literal_blocks,
            expected_blocks
        );
    }

    check("", &[""]);
    check("*", &["", ""]);
    check("*.jpg", &["", ".jpg"]);
    check("original_*.*", &["original_", ".", ""]);
    check("**.*", &["", "", ".", ""]);
}

impl Display for GlobStarPattern {
    fn fmt(&self, format: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(format, "{}", self.literal_blocks.join("*"))
    }
}

#[test]
fn test_display() {
    for pattern in [
        "",
        "hello",
        "*",
        "*hello",
        "**",
        "*hello**",
        "text*.png",
        "text*",
    ] {
        assert_eq!(GlobStarPattern::from(pattern).to_string(), pattern);
    }
}
