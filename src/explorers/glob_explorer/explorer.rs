use crate::explorers::glob_explorer::extend::ExtendedGlobPatternBuilder;
use crate::explorers::Explorer;
use crate::watched_fs::WatchedFS;

#[derive(Debug)]
pub struct GlobExplorer {
    patterns: Vec<String>,
}

/// An explorer that uses extended glob patterns to find paths on the file system.
///
/// From the glob docs: https://docs.rs/glob/latest/glob/struct.Pattern.html
///
/// > ? matches any single character.
/// > * matches any (possibly empty) sequence of characters.
/// > ** matches the current directory and arbitrary subdirectories. This sequence must form a single path component,
/// >    so both **a and b** are invalid and will result in an error. A sequence of more than two consecutive *
/// >    characters is also invalid.
/// > [...] matches any character inside the brackets. Character sequences can also specify ranges of characters, as
/// >    ordered by Unicode, so e.g. [0-9] specifies any character between 0 and 9 inclusive. An unclosed bracket is invalid.
/// > [!...] is the negation of [...], i.e. it matches any characters not in the brackets.
/// > The metacharacters ?, *, [, ] can be matched by using brackets (e.g. [?]). When a ] occurs immediately following
/// >    [ or [! then it is interpreted as being part of, rather then ending, the character set, so ] and NOT ] can be
/// >    matched by []] and [!]] respectively. The - character can be specified inside a character sequence pattern by
/// >    placing it at the start or the end, e.g. [abc-].
///
/// There is also extended support for disjunctive subpatterns using {sub1,sub2} syntax.
impl Explorer for GlobExplorer {
    fn from_cli_arg(arg: &str) -> Self {
        let patterns: Vec<String> = ExtendedGlobPatternBuilder::from_pattern(arg)
            .build()
            .into_iter()
            .collect();

        for pattern in &patterns {
            if let Err(error) = glob::Pattern::new(pattern) {
                panic!(
                    "Glob pattern from '{arg}' is invalid: '{}'",
                    error.to_string()
                );
            }
        }

        return Self { patterns };
    }

    fn explore(&self, watched_fs: &mut WatchedFS) {
        for pattern in self.patterns.iter() {
            for path in glob::glob(pattern).unwrap().filter_map(Result::ok) {
                watched_fs.find(&path);
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
    #[case("**a")]
    #[case("a**")]
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
            vec!["a.txt", "bb.yaml", "ccc.txt"],
            "*.txt",
            vec!["a.txt", "ccc.txt"],
        );
    }

    #[test]
    fn given_star_pattern_when_explore_then_does_not_match_slashes() {
        absolute_fs_test(
            vec!["a.txt", "nested/b.txt", "nested/very/deeply/c.txt"],
            "*.txt",
            vec!["a.txt"],
        );
    }

    #[test]
    fn given_question_operator_when_explore_then_substitutes_any_single_character() {
        absolute_fs_test(
            vec!["cat.txt", "dog.txt", "snake.txt"],
            "???.txt",
            vec!["cat.txt", "dog.txt"],
        );
    }

    #[test]
    fn given_positive_square_brackets_when_explore_then_matches_any_character_in_brackets() {
        absolute_fs_test(
            vec!["a.txt", "b.txt", "bb.txt", "c.txt"],
            "[ab].txt",
            vec!["a.txt", "b.txt"],
        );
    }

    #[test]
    fn given_negated_square_brackets_when_explore_then_matches_any_character_not_in_brackets() {
        absolute_fs_test(
            vec!["a.txt", "b.txt", "bb.txt", "c.txt"],
            "[!ab].txt",
            vec!["c.txt"],
        );
    }

    #[test]
    fn given_nested_directories_when_explore_then_finds_directories_too() {
        absolute_fs_test(vec!["a.txt", "nested/b.txt"], "nested", vec!["nested"]);
    }

    #[test]
    fn given_double_star_when_explore_then_searches_subdirectories() {
        absolute_fs_test(
            vec!["a.txt", "nested/b.txt", "nested/very/deeply/c.txt"],
            "nested/**/*.txt",
            vec!["nested/b.txt", "nested/very/deeply/c.txt"],
        );
    }

    #[test]
    fn given_extended_glob_pattern_when_explore_then_finds_all_matches() {
        absolute_fs_test(
            vec!["config.yml", "config.yaml"],
            "config.{yml,yaml}",
            vec!["config.yml", "config.yaml"],
        );
    }

    #[test]
    fn given_relative_glob_pattern_when_explore_then_finds_relative_matches() {
        let mut watched_fs = WatchedFS::new(10);

        // 'cargo test' will always run from the root of the project, alongside the Cargo.toml file
        let explorer = GlobExplorer::from_cli_arg("src/jfswatch.rs");
        explorer.explore(&mut watched_fs);

        let explored_paths: Vec<String> = watched_fs.paths().map(|p| p.to_string()).collect();
        assert!(
            explored_paths.contains(&"src/jfswatch.rs".to_string()),
            "Explored exactly: {:?}",
            explored_paths
        );
    }
}
