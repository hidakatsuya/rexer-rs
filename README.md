# Rexer

A Rust rewrite of the [Rexer](https://github.com/hidakatsuya/rexer) Ruby gem - a Redmine Extension (Plugin and Theme) manager.

[![CI](https://github.com/hidakatsuya/rexer-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/hidakatsuya/rexer-rs/actions/workflows/ci.yml)

## Features

Rexer is a command-line tool for managing Redmine Extensions (Plugins and Themes). It allows you to:

- Define extensions in a JSON configuration file
- Install, uninstall, update, and switch between different sets of extensions
- Support for Git and GitHub repositories with branch/tag/commit specification
- Environment-based configuration management
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

This creates a `.extensions.json` file where you can define your extensions:

```json
{
  "environments": {
    "default": [
      {
        "name": "redmine_issues_panel",
        "extension_type": "Plugin",
        "source": {
          "source_type": "GitHub",
          "url": "redmica/redmine_issues_panel",
          "reference": "v1.0.2"
        },
        "hooks": null
      }
    ],
    "stable": [
      {
        "name": "bleuclair_theme",
        "extension_type": "Theme",
        "source": {
          "source_type": "Git",
          "url": "https://github.com/farend/redmine_theme_farend_bleuclair.git",
          "reference": "master"
        },
        "hooks": null
      }
    ]
  }
}
```

Then install the extensions:

```bash
rex install
```

## Commands

- `rex init` - Create a new .extensions.json file
- `rex install [env]` - Install extensions for the specified environment
- `rex uninstall` - Uninstall all extensions
- `rex state` - Show current state of installed extensions
- `rex envs` - List all environments and their extensions
- `rex update [extensions...]` - Update extensions to latest versions
- `rex switch [env]` - Switch to a different environment
- `rex reinstall [extension]` - Reinstall a specific extension
- `rex edit` - Edit the configuration file
- `rex version` - Show version information

### Command Options

- `-v, --verbose` - Detailed output
- `-q, --quiet` - Minimal output

## Configuration

### Environment Variables

- `REXER_COMMAND_PREFIX` - Prefix for commands (e.g., `docker compose exec -T app`)
- `EDITOR` - Editor to use for `rex edit` command (default: vim)

### Extension Types

- `Plugin` - Redmine plugins (installed in `plugins/` directory)
- `Theme` - Redmine themes (installed in `public/themes/` directory)

### Source Types

- `Git` - Direct Git repository URL
- `GitHub` - GitHub repository (format: `owner/repo`)

### Reference Types

- `branch` - Git branch name
- `tag` - Git tag name  
- `commit` - Git commit hash

## Differences from Ruby Version

- Configuration uses JSON format instead of Ruby DSL for simplicity
- Hooks are not yet implemented
- Some advanced features may be missing

## Development

### Requirements

- Rust 1.70 or later
- Git
- OpenSSL development libraries

### Building

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Linting

```bash
cargo clippy
cargo fmt
```

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

Bug reports and pull requests are welcome on GitHub at https://github.com/hidakatsuya/rexer-rs.
