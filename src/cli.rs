use clap::{ArgAction, Parser};

#[derive(Debug, Parser)]
#[command(author, long_about = None, about = r#"
# JFSWatch

Justin's file system watching program.

When some path of interest on the file system changes, run a specified command.
[Project repository on GitHub](https://github.com/just1ngray/jfswatch).

## About

Run a command when watched files change. Files can be given as exact paths or
extended glob patterns. The program will check for mtime, new file, or deleted
file changes every `interval` seconds. If a change is detected, the program
will execute the specified command and sleep for `sleep` seconds before
resuming standard interval checks.

## Examples

### Simple Example

Restart the `my-program.service` systemd service when any configuration file
inside `/etc/my-program` changes, or when the binary used by the service is
updated.

JFSWatch will check for changes every 0.5 seconds, and sleep for 10 seconds
after restarting the service.

```shell
$ jfswatch \
    --interval 0.5 \
    --sleep 10.0 \
    --glob '/etc/my-program/**' \
    --exact /usr/bin/my-program \
    systemctl restart my-program.service
```

> Note: for restarting systemd services, you can create a corresponding path
> unit which will automatically restart the service when the specified path(s)
> are updated. This is probably more efficient, but if you care about
> flexibility, observability, and ease-of-use, then jfswatch will help you move
> faster with more confidence.

### Full Shell Example

When you want to use powerful shell features such as pipes (`|`), redirects
(`>`), multiple commands (`&&`), or environment variables, you must quote your
command.

Note the difference between running `"echo $SHELL"` and `'echo $SHELL'`. When
double quoted, `$SHELL` will be evaluated first and then passed into jfswatch.
When single quoted, `$SHELL` passed as a raw string to jfswatch, which will be
evaluated later when the command is run. This difference is reflected in the
jfswatch logs. For this reason it is recommended to use single quotes when
using all shell features, or substitution variables in the command.

The following example will overwrite the contents of the README with the cli's
help documentation, which proved useful while updating the documentation.

```shell
$ jfswatch \
    --glob '**/*.rs' \
    --exact Cargo.toml \
    'cargo run -- -h > README.md'
```

## Extras

- It's usually best to use single quotes when accessing full shell features.
  Otherwise the shell will evaluate substituted variables like `$diff` before
  jfswatch can use them
- Be careful not to create a loop where jfswatch watches a file that is
  modified by the command it runs. The logs will make this obvious if this
  happens, but it can still be an annoying mistake to make
- The logging level can be changed by setting the `RUST_LOG` environment
  variable to one of: `trace`, `debug`, `info`, `warn`, `error`

## Usage
```"#)]
pub struct Cli {
    /// The exact file paths to watch
    #[arg(short, long, action = ArgAction::Append)]
    pub exact: Vec<String>,

    /// The file paths to watch using extended glob patterns
    #[arg(short, long, action = ArgAction::Append)]
    pub glob: Vec<String>,

    /// Seconds to wait between each non-differing check
    #[arg(short, long, default_value_t = 0.1)]
    pub interval: f32,

    /// Seconds to sleep the program after the specified command has been executed. The program will not check for
    /// changes during this time. By default it uses the same value as `interval`
    #[arg(short, long)]
    pub sleep: Option<f32>,

    /// The command to execute when changes are detected. The command can include substitutable bash-like variables:
    /// - `$diff` or `${diff}` will be one of `new`, `deleted`, or `modified` according to the detected change.
    /// - `$path` or `${path}` will be the watched path that changed.
    /// - `$mtime` or `${mtime}` will be the last modified time of the watched path (unavailable for deleted paths).
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
