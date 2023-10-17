use std::{
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::Context;

use crate::glob_star_pattern::GlobStarPattern;

/// Source path pattern. Acts like [glob](https://en.wikipedia.org/wiki/Glob_(programming))
/// but only single star (`*`) wildcard in filenames is supported.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SourcePathPattern {
    directory: PathBuf,
    filename_pattern: GlobStarPattern,
}

impl SourcePathPattern {
    /// Number of '*'-wildcards in the filename pattern
    ///
    /// # Examples
    /// ```
    /// use mmv_lib::SourcePathPattern;
    /// use std::str::FromStr;
    /// let pattern = SourcePathPattern::from_str("rust/version-*.*").unwrap();
    /// assert_eq!(pattern.wildcards_number(), 2);
    /// ```
    pub fn wildcards_number(&self) -> usize {
        self.filename_pattern.wildcards_number()
    }

    /// Returns `Vec` of file paths matched by the pattern with the corresponding matching
    /// information, i.e. data returned by `GlobStarPattern::match_string`.
    ///
    /// # Exmaples
    /// ```
    /// use mmv_lib::SourcePathPattern;
    /// use std::{str::FromStr, path::{Path, PathBuf}};
    /// let shells = SourcePathPattern::from_str("bin/*sh")
    ///     .unwrap()
    ///     .matching_files(&Path::new("/usr"))
    ///     .unwrap();
    /// println!("{shells:?}");
    /// assert!(
    ///     shells.contains(&(PathBuf::from("bin/bash"), vec!["ba".to_string()]))
    /// )
    /// ```
    pub fn matching_files(
        &self,
        working_directory: &Path,
    ) -> anyhow::Result<Vec<(PathBuf, Vec<String>)>> {
        let mut result = vec![];
        let directory_path = working_directory.join(self.directory.clone());
        for dir_entry in std::fs::read_dir(directory_path.clone()).context(format!(
            "Failed to read {:#?} directory content",
            directory_path
        ))? {
            let entry_unwrapped = dir_entry.context("Failed to read entry")?;
            let metadata = entry_unwrapped.metadata().context(format!(
                "Failed to get metadata for {:#?}",
                entry_unwrapped.path()
            ))?;
            if metadata.is_file() {
                if let Some(filaname) = entry_unwrapped.file_name().to_str() {
                    let match_result = self.filename_pattern.match_string(filaname);
                    if let Some(match_info) = match_result {
                        result.push((
                            self.directory.join(entry_unwrapped.file_name()),
                            match_info.into_iter().map(str::to_string).collect(),
                        ));
                    }
                }
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod test_getting_matching_files {
    use super::SourcePathPattern;
    use std::{fs::File, str::FromStr};
    use tempdir::TempDir;

    #[test]
    fn it_works() -> anyhow::Result<()> {
        let working_directory =
            TempDir::new("gallifrey").expect("Failed to create a temporary directory");
        let directory_with_files = working_directory.path().join("timelords");
        std::fs::create_dir(directory_with_files.as_path())
            .expect("Failed to create a nested temporary directory");

        const DOCTORS_COUNT: u32 = 13;
        const MASTERS_COUNT: u32 = 9;

        for doctor_index in 1..=DOCTORS_COUNT {
            let _ = File::create(directory_with_files.join(doctor_index.to_string() + ".doc"));
        }
        std::fs::create_dir(directory_with_files.join("0.doc"))
            .expect("Failed to create a directory");

        for master_index in 1..=MASTERS_COUNT {
            let _ = File::create(directory_with_files.join(master_index.to_string() + ".master"));
        }

        let any_doctor_pattern = SourcePathPattern::from_str("timelords/*.doc").unwrap();
        assert!(any_doctor_pattern
            .matching_files(directory_with_files.as_path())
            .is_err());

        let mut matched_files = any_doctor_pattern
            .matching_files(working_directory.path())
            .expect("Shouldn't fail");
        matched_files.sort();
        let mut expected_files = (1..=DOCTORS_COUNT)
            .map(|doctor_index| {
                (
                    ("timelords/".to_string() + doctor_index.to_string().as_str() + ".doc").into(),
                    vec![doctor_index.to_string()],
                )
            })
            .collect::<Vec<_>>();
        expected_files.sort();

        assert_eq!(matched_files, expected_files);
        Ok(())
    }
}

impl FromStr for SourcePathPattern {
    type Err = &'static str;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let (directory_str, filename_pattern_str) = string.split_at(
            string
                .rfind('/')
                .map_or(0, |slash_position| slash_position + 1),
        );
        if directory_str.contains('*') {
            return Err("'*'-wildcard can only appear in a filename");
        }
        Ok(Self {
            directory: PathBuf::from(directory_str),
            filename_pattern: GlobStarPattern::from(filename_pattern_str),
        })
    }
}

#[test]
fn test_source_path_pattern_from_str() {
    assert_eq!(
        SourcePathPattern::from_str("doctor/in/blue/box/*.tardis"),
        Ok(SourcePathPattern {
            directory: PathBuf::from("doctor/in/blue/box/"),
            filename_pattern: GlobStarPattern::from("*.tardis")
        })
    );

    assert_eq!(
        SourcePathPattern::from_str("master*dalek"),
        Ok(SourcePathPattern {
            directory: PathBuf::default(),
            filename_pattern: GlobStarPattern::from("master*dalek")
        })
    );

    assert_eq!(
        SourcePathPattern::from_str("/from_root.*"),
        Ok(SourcePathPattern {
            directory: PathBuf::from("/"),
            filename_pattern: GlobStarPattern::from("from_root.*")
        })
    );

    assert!(SourcePathPattern::from_str("b*d/pattern").is_err());
}

impl Display for SourcePathPattern {
    fn fmt(&self, format: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.directory.to_str() {
            Some(directory_str) => write!(format, "{directory_str}{}", self.filename_pattern),
            None => Err(std::fmt::Error),
        }
    }
}

#[test]
fn test_display() {
    for pattern in [
        "",
        "/",
        "/path/to/file#1",
        "/hello/world",
        "empty/filename/",
        "path/*.png",
    ] {
        assert_eq!(
            SourcePathPattern::from_str(pattern).unwrap().to_string(),
            pattern
        );
    }
}
