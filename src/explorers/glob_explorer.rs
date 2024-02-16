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
