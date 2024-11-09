# affected

A tool to find affected files or projects in a git repository.

*Coming soon*

## Usage

The format of the command is:

```bash
affected [OPTIONS] <COMMAND>
```

### Example

For the feature branch checked out, and the main branch is `develop`:

```bash
# List all affected files in the current repository
affected --main=develop list all

# List all affected files in a different repository
affected --repo=/path/to/repo --main=develop list all
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