# JFSWatch

Justin's file system watching program.

When some path of interest on the file system changes, run a specified command.

## About

Run a command when watched files change. Files can be given as exact paths or
extended glob patterns. The program will check for mtime, new file, or deleted
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
jfswatch logs. For this reason it is recommended to use single quotes when
using all shell features, or substitution variables in the command.

```shell
$ jfswatch \
    --exact Cargo.toml \
    'echo running command in $SHELL && echo $(date) >> Cargo.toml_was_modified.txt'
```

## Usage
```

Usage: jfswatch [OPTIONS] <CMD>...

Arguments:
  <CMD>...  The command to execute when changes are detected. The command can include substitutable bash-like variables: `$diff` or `${diff}` will be one of `new`, `deleted`, or `modified` according to the detected change. `$path` or `${path}` will be the watched path that changed. `$mtime` or `${mtime}` will be the last modified time of the watched path (unavailable for deleted paths)

Options:
  -e, --exact <EXACT>        The exact file paths to watch
  -g, --glob <GLOB>          The file paths to watch using extended glob patterns
  -r, --regex <REGEX>        The file paths to watch using anchored regex patterns (NOT IMPLEMENTED YET!)
  -i, --interval <INTERVAL>  Seconds to wait between each non-differing check [default: 0.1]
  -s, --sleep <SLEEP>        Seconds to sleep the program after the specified command has been executed. The program will not check for changes during this time. By default it uses the same value as `interval`
  -h, --help                 Print help