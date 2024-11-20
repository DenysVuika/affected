# affected

A tool to find affected files or projects in a git repository and run commands on them.

## Features

- determine affected files or projects for a git repository
- view affected files or projects
- run commands on affected files or projects
    - support `.env` files for the commands
- supports [Nx](https://nx.dev/) monorepos
    - `implicitDependencies` via the `project.json` files

## Installation

### From Crates.io

This is the recommended way to install the tool.

```bash
cargo install affected
```

### From pre-built binaries

Download the latest release from the [releases page](https://github.com/DenysVuika/affected/releases)

Supported platforms:

- Linux (x86_64)
- Linux (aarch64)
- macOS (x86_64, unsigned)

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

### Commands

- `init` - Initialize the configuration file
- `view` - View affected files or projects
    - `files` - List affected files
    - `projects` - List affected projects
    - `tasks` - List defined tasks
- `run [task|glob]` - Run a task on affected files or projects

For more information on a command, use the `help` command.

### Example

For the feature branch checked out, and the main branch is `develop`:

```bash
# List all affected files in the current repository
affected --base=develop view files

# List all affected projects in a different repository
affected --repo=/path/to/repo --base=develop view projects
```

## Tasks

Tasks can be defined in the `.affected.yml` file to run commands on affected files.

```yaml
base: develop

tasks:
  - name: lint
    description: Runs eslint for all affected files
    patterns: [ '*.{ts,tsx,js,jsx}' ]
    separator: ' ' # Optional separator for the files list
    commands: [ 'npx eslint {files}' ]

    # Running eslint for affected files
    # commands: [ 'npx eslint {files}' ]
```

The `name` field is the name of the task.  
The `patterns` field is an array of file patterns to match.  
The `separator` field is an optional separator for the files list.  
The `commands` field is an array of commands to run on the affected files.  
The `{files}` placeholder is replaced with the list of affected files.

Alternative formatting:

```yaml
base: main
tasks:
  - name: prettier
    description: Runs prettier for all affected files
    patterns:
      - '*.ts'
      - '*.tsx'
      - '*.js'
      - '*.jsx'
    commands:
      - echo {files}
      - npx prettier --check {files}
```

Examples:

```bash
# view the tasks defined in the configuration file
affected view tasks

# run the task named 'lint'
affected run lint

# run tasks by a glob pattern (e.g. 'project:1', 'project:2', etc.)
affected run "project:*"
```

### File Separators

The separator for the list of files can be set using the `separators` task field.
This is useful when the command requires a different separator than the default.

For example, the `karma` task requires a comma-separated list of files.

```yaml
tasks:
  - name: karma
    description: Runs karma for all affected files
    patterns: [ '*.spec.ts' ]
    separator: ',' # Comma-separated list
    commands: [ 'npx karma start --include {files}' ] 
```

The default separator is a space.

### Using Glob Patterns

Glob patterns can be used to run multiple tasks on different file types.

```yaml
# .affected.yml
base: develop
tasks:
  - name: echo:1
    description: Echoes the affected files (example)
    patterns:
      - '*'
    commands:
      - echo 'one'
  - name: echo:2
    description: Echoes the affected files (example)
    patterns:
      - '*'
    commands:
      - echo 'two'
```

Run the tasks:

```bash
affected run "echo:*"
```

The output is:

```bash
one
two
```

### Environment Variables

Environment variables can be set in the `.env` file in the root of the repository.

```bash
# .env
NODE_ENV=development
E2E_HOST="http://localhost"
E2E_PORT=4200
```

The variables are passed to the commands.

```yaml
tasks:
  - name: demo
    description: Passes environment variables to the command
    patterns: [ '*.{ts,tsx,js,jsx}' ]
    commands: [ 'echo ${E2E_HOST}:${E2E_PORT}' ]
```

Modify some source files and run the task:

```bash
affected run demo
```

The output is `http://localhost:4200`.

## Developing

### Log Levels

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

### Local Runs

You can compile and run the application locally for testing.

```shell
cargo build
sudo ln -s $(pwd)/target/debug/affected /usr/local/bin/affected

# Run the application
cd /path/to/repo
affected view files

# Full debug output
LOG_LEVEL=DEBUG affected run lint
```

## Recipes

Below are some examples of tasks that can be defined in the `.affected.yml` file.

### Run ESLint on affected files

```yaml
tasks:
  - name: lint
    description: Runs eslint for all affected files
    patterns: [ '*.{ts,tsx,js,jsx}' ]
    commands: [ 'npx eslint {files}' ]
```

### Run Prettier on affected files

```yaml
tasks:
  - name: prettier
    description: Runs prettier for all affected files
    patterns: [ '*.{ts,tsx,js,jsx}' ]
    commands: [ 'npx prettier --check {files}' ]
```

### Run Karma on affected files

```yaml
tasks:
  - name: karma
    description: Runs karma for all affected files
    patterns: [ '*.spec.ts' ]
    separator: ',' # Comma-separated list
    commands: [ 'npx karma start --include {files}' ] 
```

### Run Jest on affected files

```yaml
tasks:
  - name: jest
    description: Runs jest for all affected files
    patterns: [ '*.spec.ts' ]
    commands: [ 'npx jest --findRelatedTests --passWithNoTests -c <jest-config> {files}' ]
```

Where `<jest-config>` is the path to the Jest configuration file.