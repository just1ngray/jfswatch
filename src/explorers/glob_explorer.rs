use std::fs;

use crate::explorers::Explorer;
use crate::watched_fs::WatchedFS;

#[derive(Debug)]
pub struct GlobExplorer {
    pattern: String,
}

/// From the glob docs: https://docs.rs/glob/latest/glob/struct.Pattern.html
///
/// ? matches any single character.
/// * matches any (possibly empty) sequence of characters.
/// ** matches the current directory and arbitrary subdirectories. This sequence must form a single path component, so both **a and b** are invalid and will result in an error. A sequence of more than two consecutive * characters is also invalid.
/// [...] matches any character inside the brackets. Character sequences can also specify ranges of characters, as ordered by Unicode, so e.g. [0-9] specifies any character between 0 and 9 inclusive. An unclosed bracket is invalid.
/// [!...] is the negation of [...], i.e. it matches any characters not in the brackets.
/// The metacharacters ?, *, [, ] can be matched by using brackets (e.g. [?]). When a ] occurs immediately following [ or [! then it is interpreted as being part of, rather then ending, the character set, so ] and NOT ] can be matched by []] and [!]] respectively. The - character can be specified inside a character sequence pattern by placing it at the start or the end, e.g. [abc-].
impl Explorer for GlobExplorer {
    fn from_cli_arg(arg: &str) -> Self {
        return match glob::Pattern::new(arg) {
            Ok(_) => Self {
                pattern: arg.to_string(),
            },
            Err(error) => panic!("{}", error.to_string()),
        };
    }

    fn explore(&self, watched_fs: &mut WatchedFS) {
        for path in glob::glob(&self.pattern).unwrap().filter_map(Result::ok) {
            if let Ok(metadata) = fs::metadata(&path) {
                let mtime = metadata
                    .modified()
                    .expect("mtime is not supported on your platform");
                watched_fs.found(path.to_string_lossy().to_string(), mtime);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use rstest::rstest;
    use tempfile::tempdir_in;

    use super::*;
    use crate::test_utils::utils::make_files;

    #[rstest]
    #[case("[")]
    #[should_panic]
    fn given_invalid_glob_pattern_when_new_glob_explorer_then_panics(#[case] pattern: &str) {
        GlobExplorer::from_cli_arg(pattern);
    }

    #[test]
    fn given_simple_pattern_when_explore_then_finds_exact_match() {
        let tmp = tempdir_in(".").unwrap();
        let basedir = tmp.path().to_owned();
        let mut watched_fs = WatchedFS::new(3);
        make_files(&basedir, vec!["a.txt", "b.txt", "c.txt"]);

        let glob_pattern = format!("{}/b.txt", basedir.to_string_lossy());
        let explorer = GlobExplorer::from_cli_arg(&glob_pattern);
        explorer.explore(&mut watched_fs);

        assert_eq!(watched_fs.len(), 1);
        assert_eq!(
            watched_fs
                .paths()
                .map(|p| p.to_string())
                .collect::<HashSet<String>>(),
            HashSet::from([glob_pattern])
        );
    }

    #[test]
    fn given_star_pattern_when_explore_then_finds_matches() {
        let tmp = tempdir_in(".").unwrap();
        let basedir = tmp.path().to_owned();
        let mut watched_fs = WatchedFS::new(3);
        let fullpaths = make_files(&basedir, vec!["a.txt", "b.yaml", "c.txt"]);
        let a_txt = &fullpaths[0];
        let b_txt = &fullpaths[2];

        let glob_pattern = format!("{}/*.txt", basedir.to_string_lossy());
        let explorer = GlobExplorer::from_cli_arg(&glob_pattern);
        explorer.explore(&mut watched_fs);

        assert_eq!(watched_fs.len(), 2);
        assert_eq!(
            watched_fs
                .paths()
                .map(|p| p.to_string())
                .collect::<HashSet<String>>(),
            HashSet::from([
                a_txt.to_string_lossy().to_string(),
                b_txt.to_string_lossy().to_string()
            ])
        );
    }
}
