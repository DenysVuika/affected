# affected

A tool to find affected files or projects in a git repository.

## Installation

### From Crates.io

```bash
cargo install affected
```

## Usage

The format of the command is:

```bash
Usage: affected [OPTIONS] <COMMAND>

Commands:
  init      
  files     
  projects  
  help      Print this message or the help of the given subcommand(s)

Options:
      --repo <REPO>  Optional repo path, defaults to current directory
      --base <BASE>  Base of the current branch (usually main). Falls back to 'main' or 'master' if not provided
  -h, --help         Print help
```

### Example

For the feature branch checked out, and the main branch is `develop`:

```bash
# List all affected files in the current repository
affected --base=develop files list

# List all affected files in a different repository
affected --repo=/path/to/repo --base=develop files list
```

## Init

The `init` command initializes the repository for use with the `affected` tool.

```bash
affected init
# or
affected --base=develop init
```

## Log Levels

The log level can be set using the `LOG_LEVEL` environment variable.

```bash
export LOG_LEVEL=DEBUG
affected [OPTIONS] <COMMAND>
```

The following log levels are available:

- `TRACE`
- `DEBUG`
- `INFO`
- `WARN`
- `ERROR`

The default log level is `INFO`.

## Local Development

Linking the binary for global use:

```shell
cargo build
sudo ln -s $(pwd)/target/debug/affected /usr/local/bin/affected
```