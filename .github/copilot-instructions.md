# Copilot Instructions for rexer-rs

## Project Overview

This is a Rust rewrite of the original Ruby [rexer](https://github.com/hidakatsuya/rexer) gem, providing a fast, cross-platform CLI tool for managing Redmine extensions (plugins and themes).

## Key Principles

### Architecture & Design
- **Minimal Changes**: Always make the smallest possible changes to achieve the goal
- **Type Safety**: Leverage Rust's type system for configuration validation and error handling
- **Modular Design**: Maintain clean separation of concerns (CLI, commands, config, git, extensions)
- **Error Handling**: Use `anyhow` and `thiserror` for robust error management
- **Performance**: Focus on fast startup and execution times

### Configuration Format
- **Config Files**: Use YAML format (`.extensions.yml`) for human readability and broad accessibility
- **Lock Files**: Use JSON format (`.extensions.lock`) for machine precision and performance
- **Git References**: Use explicit `branch`, `tag`, and `commit` fields instead of generic `reference`
- **No Environment Support**: Simplified architecture without environment concepts from Ruby version

### Directory Structure
- **Plugins**: Install to `plugins/` directory
- **Themes**: Install to `themes/` directory (not `public/themes/`)

### CLI Behavior
- **Default Command**: Runs `install` when no subcommand specified
- **Ruby Compatibility**: All commands should match the original Ruby rexer functionality
- **Output Modes**: Support verbose (`-v`) and quiet (`-q`) modes

## Development Guidelines

### Code Quality
- **Testing**: Always maintain comprehensive integration tests that actually execute the `rex` command
- **Linting**: Use `cargo clippy` and `cargo fmt` for code quality
- **CI/CD**: Use only official GitHub Actions (no third-party actions for security)

### Git Operations
- **GitHub Shortcuts**: Support `owner/repo` format for GitHub repositories
- **Git References**: Full support for branch, tag, and commit references
- **State Tracking**: Maintain proper lock file state with commit hashes and timestamps

### Install Command Logic
The install command must implement sophisticated diff-based logic:
- **Diff Calculation**: Compare lock file state with current configuration
- **Smart Updates**: Only install/update/uninstall what has actually changed
- **Source Change Detection**: Automatically detect when Git references change
- **State Consistency**: Maintain proper lock file state for reliable dependency tracking

### Commands Implementation
Ensure all commands match Ruby rexer behavior:
- **init**: Creates `.extensions.yml` with proper YAML structure
- **install**: Complete diff-based logic (added, removed, source_changed)
- **uninstall**: Removes all extensions and deletes lock file
- **state**: Shows version and groups extensions by type
- **update**: Updates specific extensions or all if no names provided
- **reinstall**: Reinstalls specific extension (uninstall + install)
- **edit**: Opens config file in $VISUAL or $EDITOR

### Excluded Features
- **Hooks**: Ruby-style hooks are not needed in the Rust version
- **Environments**: Environment support is not implemented
- **`rex envs` command**: Not needed in this implementation

## Testing Strategy

### Integration Tests
- Test actual `rex` command execution
- Use real GitHub repositories for testing (e.g., octocat/Hello-World)
- Cover all CLI functionality and edge cases
- Use temporary directories for test isolation

### Error Handling
- Test invalid configuration formats
- Test missing files and lock files
- Test non-existent extensions
- Test command failures

## File Organization

### Source Structure
- `src/main.rs` - Entry point
- `src/cli.rs` - Command-line interface using `clap`
- `src/commands.rs` - Command implementations
- `src/config.rs` - Configuration and lock file handling
- `src/extension.rs` - Extension types and structures
- `src/git.rs` - Git operations
- `src/error.rs` - Error types and handling

### Test Structure
- `tests/cli_tests.rs` - Integration tests for CLI commands
- Focus on real command execution rather than unit tests

## Dependencies

### Core Dependencies
- `clap` - Command-line argument parsing
- `serde` and `serde_yaml` - Configuration serialization
- `git2` - Git operations
- `tokio` - Async runtime for future extensibility
- `anyhow` and `thiserror` - Error handling

### Development Dependencies
- `assert_cmd` - Command testing
- `tempfile` - Temporary directories for tests

## Future Considerations

- Maintain compatibility with Ruby rexer's core functionality
- Keep configuration format simple and accessible
- Prioritize performance and ease of deployment
- Consider adding features that benefit from Rust's strengths (parallelism, safety)