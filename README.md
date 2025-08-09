# Rexer

A fast, cross-platform CLI tool for managing Redmine Extensions (Plugins and Themes).

[![CI](https://github.com/hidakatsuya/rexer-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/hidakatsuya/rexer-rs/actions/workflows/ci.yml)

## Features

Rexer is a command-line tool for managing Redmine Extensions (Plugins and Themes). It allows you to:

- Define extensions in a YAML configuration file
- Install, uninstall, update, and manage extensions
- Support for Git and GitHub repositories with branch/tag/commit specification
- Cross-platform support (Linux, macOS)

## Installation

### Pre-compiled binaries

Download the latest release from the [releases page](https://github.com/hidakatsuya/rexer-rs/releases).

### Build from source

```bash
git clone https://github.com/hidakatsuya/rexer-rs.git
cd rexer-rs
cargo build --release
```

The binary will be available at `target/release/rex`.

## Usage

Run the following command in the root directory of your Redmine application:

```bash
rex init
```

This creates a `.extensions.yml` file where you can define your extensions:

```yaml
plugins:
  # Example plugin from GitHub
  - name: redmine_issues_panel
    github:
      repo: "redmica/redmine_issues_panel"
      tag: "v1.0.2"

themes:
  # Example theme from Git repository  
  - name: bleuclair_theme
    git:
      url: "https://github.com/farend/redmine_theme_farend_bleuclair.git"
      branch: "master"
```

Then install the extensions:

```bash
rex install
```

## Commands

- `rex init` - Create a new .extensions.yml file
- `rex install` - Install extensions defined in .extensions.yml (compares config with lock file)
- `rex uninstall` - Uninstall all extensions
- `rex state` - Show current state of installed extensions
- `rex update [extensions...]` - Update extensions to latest versions based on sources in lock file
- `rex reinstall [extension]` - Reinstall a specific extension
- `rex edit` - Edit the configuration file
- `rex version` - Show version information

### Command Options

- `-v, --verbose` - Detailed output
- `-q, --quiet` - Minimal output

### Install vs Update

- **`rex install`** - Compares your `.extensions.yml` configuration with the current `.extensions.lock` file and installs, updates, or removes extensions as needed to match the configuration.
- **`rex update`** - Updates specific extensions (or all if none specified) to their latest versions based on the source configuration stored in the `.extensions.lock` file. This only looks at the lock file and does not compare with `.extensions.yml`.

## Configuration

### Extension Types

- `Plugin` - Redmine plugins (installed in `plugins/` directory)
- `Theme` - Redmine themes (installed in `themes/` directory)

### Source Types

- `Git` - Direct Git repository URL
- `GitHub` - GitHub repository (format: `owner/repo`)

### Reference Types

- `branch` - Git branch name
- `tag` - Git tag name  
- `commit` - Git commit hash

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

Bug reports and pull requests are welcome on GitHub at https://github.com/hidakatsuya/rexer-rs.
