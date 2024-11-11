# affected

A tool to find affected files or projects in a git repository and run commands on them.

## Installation

### From Crates.io

```bash
cargo install affected
```

## Setup

The `init` command initializes the repository for use with the `affected` tool.

```bash
cd /path/to/repo

# with the default base branch
affected init

# or with a different base branch
affected --base=develop init
```

This command creates a `.affected.yml` file in the root of the repository.

### Configuration

```yaml
# .affected.yml
base: main
```

## Usage

The format of the command is:

```bash
Usage: affected [OPTIONS] <COMMAND>

Commands:
  init  Initialize the configuration file
  view  View affected files or projects
  help  Print this message or the help of the given subcommand(s)

Options:
      --repo <REPO>  Optional repo path, defaults to current directory
      --base <BASE>  Base of the current branch (usually main). Falls back to 'main' or 'master' if not provided
  -h, --help         Print help
```

### Example

For the feature branch checked out, and the main branch is `develop`:

```bash
# List all affected files in the current repository
affected --base=develop view files

# List all affected projects in a different repository
affected --repo=/path/to/repo --base=develop view projects
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