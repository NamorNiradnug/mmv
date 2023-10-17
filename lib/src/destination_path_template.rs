use std::{path::PathBuf, str::pattern::Pattern};

/// Destination path template. Can contain special markers such as `#1`, `#2`, etc.
/// These markers are to be replaced by another symbols, for instance by fragments of source
/// filenames matched by wildcards.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DestinationPathTemplate<'a> {
    /// Directory containing a destination file
    pub directory: PathBuf,
    markers: Vec<u8>,
    literal_blocks: Vec<&'a str>,
}

impl<'a> DestinationPathTemplate<'a> {
    /// Compiles pattern. Markers in `path_pattern` are ignored anywhere but the filename, i.e. in directories names.
    /// `max_marker_index` is typically a number of wildcards in a corresponding `SourcePathPattern`.
    /// A marker with the greatest possible (i.e. not greater than `max_marker_index`) index will be used in case of ambiguity.
    /// For example, if `max_marker_index` is `10`, then `"#100"` will be treated as marker `#10` and literal `"0"`.
    /// See [`substitute`][DestinationPathPattern::substitute] method for more exmaples.
    ///
    /// # Exmaples
    /// ```
    /// use mmv_lib::DestinationPathTemplate;
    /// let rust_version_pattern = DestinationPathTemplate::compile("rust/version-#1-#2", 2);
    /// assert_eq!(
    ///     rust_version_pattern.directory,
    ///     std::path::PathBuf::from("rust/"),
    /// );
    /// ```
    pub fn compile(path_pattern: &'a str, max_marker_index: u8) -> Self {
        let (directory, mut filename_remainder) =
            path_pattern.split_at(path_pattern.rfind('/').map_or(0, |index| index + 1));
        let mut markers = vec![];
        let mut literal_blocks = vec![];
        let mut current_offset_in_filename = 0;
        'find_marker: while let Some(hashtag_position) =
            &filename_remainder[current_offset_in_filename..].find('#')
        {
            for marker_index in (1..=max_marker_index).rev() {
                if marker_index.to_string().is_prefix_of(
                    &filename_remainder[current_offset_in_filename + hashtag_position + 1..],
                ) {
                    literal_blocks
                        .push(&filename_remainder[..current_offset_in_filename + hashtag_position]);
                    markers.push(marker_index);
                    filename_remainder = &filename_remainder[current_offset_in_filename
                        + hashtag_position
                        + marker_index.to_string().len()
                        + 1..];
                    current_offset_in_filename = 0;
                    continue 'find_marker;
                }
            }
            current_offset_in_filename = hashtag_position + 1;
        }
        literal_blocks.push(filename_remainder);
        Self {
            directory: directory.into(),
            markers,
            literal_blocks,
        }
    }

    /// Subtitutes `fragments_values` instead of markers: `#1` is replaced by
    /// `fragments_values[0]`, etc.
    ///
    /// # Panics
    /// Panics if `fragments_values` doesn't contain enough fragments to substitute.
    ///
    /// # Examples
    /// ```
    /// use mmv_lib::DestinationPathTemplate;
    /// let rust_version_pattern = DestinationPathTemplate::compile("rust/version-#1.#2", 2);
    /// assert_eq!(
    ///     rust_version_pattern.substitute(&["1", "75"]),
    ///     std::path::PathBuf::from("rust/version-1.75")
    /// );
    ///
    /// // Note passing `1` as a `max_marker_index`
    /// let unparsed_marker = DestinationPathTemplate::compile("file#1.#2", 1);
    /// assert_eq!(
    ///     // no matter how many fragment are passed,
    ///     // markers with indices greater than 1 won't work
    ///     unparsed_marker.substitute(&["hello", "world"]),
    ///     std::path::PathBuf::from("filehello.#2")
    /// );
    /// ```
    pub fn substitute(&self, fragments_values: &[&str]) -> PathBuf {
        let mut result_filename = self.literal_blocks[0].to_string();
        for (marker_index, block) in self.markers.iter().zip(self.literal_blocks.iter().skip(1)) {
            result_filename += fragments_values[*marker_index as usize - 1];
            result_filename += block;
        }
        self.directory.join(result_filename)
    }
}

#[cfg(test)]
mod test_destination_path_pattern {
    use std::path::PathBuf;

    use super::DestinationPathTemplate;

    #[test]
    fn compile() {
        assert_eq!(
            DestinationPathTemplate::compile("file_#1_name.#2", 2),
            DestinationPathTemplate {
                directory: "".into(),
                markers: vec![1, 2],
                literal_blocks: vec!["file_", "_name.", ""]
            }
        );

        assert_eq!(
            DestinationPathTemplate::compile("#1#2#1#1#2", 2),
            DestinationPathTemplate {
                directory: "".into(),
                markers: vec![1, 2, 1, 1, 2],
                literal_blocks: vec![""; 6]
            },
        );

        assert_eq!(
            DestinationPathTemplate::compile("#1#12#123#1#123", 12),
            DestinationPathTemplate {
                directory: "".into(),
                markers: vec![1, 12, 12, 1, 12],
                literal_blocks: vec!["", "", "", "3", "", "3"]
            }
        );

        assert_eq!(
            DestinationPathTemplate::compile("path/to/file_##1.#2", 5),
            DestinationPathTemplate {
                directory: "path/to/".into(),
                markers: vec![1, 2],
                literal_blocks: vec!["file_#", ".", ""]
            }
        );

        assert_eq!(
            DestinationPathTemplate::compile("path#1/#1#2.png", 3),
            DestinationPathTemplate {
                directory: PathBuf::from("path#1/"),
                markers: vec![1, 2],
                literal_blocks: vec!["", "", ".png"]
            }
        );

        assert_eq!(
            DestinationPathTemplate::compile("/absolute/path/#20.#2", 20),
            DestinationPathTemplate {
                directory: PathBuf::from("/absolute/path/"),
                markers: vec![20, 2],
                literal_blocks: vec!["", ".", ""]
            }
        );

        assert_eq!(
            DestinationPathTemplate::compile("/file_in_root#1.png", 1),
            DestinationPathTemplate {
                directory: PathBuf::from("/"),
                markers: vec![1],
                literal_blocks: vec!["file_in_root", ".png"]
            }
        );
    }

    #[test]
    #[should_panic]
    fn substitute_out_of_range() {
        DestinationPathTemplate::compile("#1", 1).substitute(&[]);
    }

    #[test]
    fn substitute() {
        let pattern = DestinationPathTemplate::compile("dir/file_#2.#1", 2);
        assert_eq!(
            pattern.substitute(&["meow", "oink"]),
            PathBuf::from("dir/file_oink.meow")
        );
        assert_eq!(
            pattern.substitute(&["#2", "#1"]),
            PathBuf::from("dir/file_#1.#2")
        );
    }
}
