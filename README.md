# jfswatch

Justin's file system watching program

```
Run a command when watched files change. Files can be given as exact paths or
basic glob patterns. The program will check for mtime, new file, or deleted
file changes every `interval` seconds. If a change is detected, the program
will execute the specified command and sleep for `sleep` seconds before
resuming standard interval checks

Usage: jfswatch [OPTIONS] <CMD>...

Arguments:
  <CMD>...  The command to execute when changes are detected

Options:
  -e, --exact <EXACT>        The exact file paths to watch
  -g, --glob <GLOB>          The file paths to watch using basic glob patterns
  -r, --regex <REGEX>        The file paths to watch using anchored regex
                             patterns
  -i, --interval <INTERVAL>  Seconds to wait between each non-differing check
                             [default: 0.1]
  -s, --sleep <SLEEP>        Seconds to sleep the program after the specified
                             command has been executed. The program will not
                             check for changes during this time. By default it
                             uses the same value as `interval`
  -h, --help                 Print help
  -V, --version              Print version
```
