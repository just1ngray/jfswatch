use clap::{ArgAction, Parser};

#[derive(Debug, Parser)]
#[command(author, long_about = None, about = r#"
# JFSWatch

Justin's file system watching program.

When some path of interest on the file system changes, run a specified command.

## About

Run a command when watched files change. Files can be given as exact paths or
basic glob patterns. The program will check for mtime, new file, or deleted
file changes every `interval` seconds. If a change is detected, the program
will execute the specified command and sleep for `sleep` seconds before
resuming standard interval checks.

The logging level can be changed by setting the `RUST_LOG` environment variable
to one of: `trace`, `debug`, `info`, `warn`, `error`.

## Examples

### Simple Example

Run `cargo test` when any Rust file changes. Check for changes every 0.5
seconds and sleep for 2.0 seconds after running the tests.

```shell
$ jfswatch \
    --interval 0.5 \
    --sleep 2.0 \
    --glob '**/*.rs' \
    --exact Cargo.toml \
    cargo test
```

### Full Shell Example

When you want to use powerful shell features such as pipes (`|`), redirects
(`>`), multiple commands (`&&`), or environment variables, you must quote your
command.

For example, each time `Cargo.toml` is modified, append the current date to a
file called `Cargo.toml_was_modified.txt` and print the `$SHELL` environment
variable used to execute that command.

Note the difference between running `"echo $SHELL"` and `'echo $SHELL'`. When
double quoted, `$SHELL` will be evaluated first and then passed into jfswatch.
When single quoted, `$SHELL` passed as a raw string to jfswatch, which will be
evaluated later when the command is run. This difference is reflected in the
jfswatch logs.

```shell
$ jfswatch \
    --exact Cargo.toml \
    'echo running command in $SHELL && echo $(date) >> Cargo.toml_was_modified.txt'
```

## Usage
```"#)]
pub struct Cli {
    /// The exact file paths to watch
    #[arg(short, long, action = ArgAction::Append)]
    pub exact: Vec<String>,

    /// The file paths to watch using basic glob patterns
    #[arg(short, long, action = ArgAction::Append)]
    pub glob: Vec<String>,

    /// The file paths to watch using anchored regex patterns (NOT IMPLEMENTED YET!)
    #[arg(short, long, action = ArgAction::Append)]
    pub regex: Vec<String>,

    /// Seconds to wait between each non-differing check
    #[arg(short, long, default_value_t = 0.1)]
    pub interval: f32,

    /// Seconds to sleep the program after the specified command has been executed. The program will not check for
    /// changes during this time. By default it uses the same value as `interval`
    #[arg(short, long)]
    pub sleep: Option<f32>,

    /// The command to execute when changes are detected
    #[arg(required = true)]
    pub cmd: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    use clap::CommandFactory;

    #[test]
    fn given_main_jfswatch_clap_command_when_debug_assert_then_clap_authors_approve() {
        return Cli::command().debug_assert();
    }

    #[test]
    fn given_cli_help_text_when_compared_against_readme_then_is_the_same() {
        let help_text = Cli::command().render_help().to_string();
        let readme = include_str!("../README.md");
        assert_eq!(
            readme.trim(),
            help_text.trim(),
            "README.md needs to be updated. Run 'cargo run -- -h > README.md'"
        );
    }
}
