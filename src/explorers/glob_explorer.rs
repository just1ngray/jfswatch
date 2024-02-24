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

    fn absolute_fs_test(files: Vec<&str>, glob_pattern: &str, expected_relative_paths: Vec<&str>) {
        let tmp = tempdir_in(".").unwrap();
        let basedir = tmp.path().to_owned();
        let mut watched_fs = WatchedFS::new(10);
        make_files(&basedir, files);

        // to use the temporary 'basedir', we must make the glob patterns relative to this directory and not
        // the current working directory. this is accomplished by translating it to an absolute path
        let glob_pattern = format!("{}/{}", basedir.to_string_lossy(), glob_pattern);
        let explorer = GlobExplorer::from_cli_arg(&glob_pattern);
        explorer.explore(&mut watched_fs);

        assert_eq!(watched_fs.len(), expected_relative_paths.len());

        let expected_absolute_paths: HashSet<String> = expected_relative_paths
            .iter()
            .map(|p| format!("{}/{}", basedir.to_string_lossy(), p))
            .collect();
        let actually_found_paths: HashSet<String> =
            watched_fs.paths().map(|p| p.to_string()).collect();
        assert_eq!(actually_found_paths, expected_absolute_paths);
    }

    #[rstest]
    #[case("[")]
    #[should_panic]
    fn given_invalid_glob_pattern_when_new_glob_explorer_then_panics(#[case] pattern: &str) {
        GlobExplorer::from_cli_arg(pattern);
    }

    #[test]
    fn given_simple_pattern_when_explore_then_finds_exact_match() {
        absolute_fs_test(vec!["a.txt", "b.txt", "c.txt"], "b.txt", vec!["b.txt"]);
    }

    #[test]
    fn given_star_pattern_when_explore_then_finds_matches() {
        absolute_fs_test(
            vec!["a.txt", "b.yaml", "c.txt"],
            "*.txt",
            vec!["a.txt", "c.txt"],
        );
    }
}
