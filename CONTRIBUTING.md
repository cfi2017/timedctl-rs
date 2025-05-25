# Contributing to timedctl-rs

Thank you for your interest in contributing to timedctl-rs! This document provides guidelines and instructions for contributing to this project.

## Code of Conduct

Please be respectful and considerate of others when contributing to this project. We aim to foster an inclusive and welcoming community.

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally: `git clone https://github.com/your-username/timedctl-rs.git`
3. Add the upstream repository: `git remote add upstream https://github.com/cfi2017/timedctl-rs.git`
4. Create a new branch for your feature or bugfix: `git checkout -b feature/your-feature-name`

## Development Workflow

### Setting up your Development Environment

1. Install Rust using [rustup](https://rustup.rs/)
2. Install development dependencies:
   ```bash
   cargo install cargo-bump
   cargo install cargo-watch
   ```
3. Install Go from [go.dev](https://go.dev/dl/) (required for go-semantic-release)
4. Install pre-commit: [pre-commit.com](https://pre-commit.com/#install)
5. Set up the git hooks: `pre-commit install`
7. Run tests to make sure everything is working: `cargo test`

### Making Changes

1. Make your changes in your feature branch
2. Write or update tests as needed
3. Pre-commit hooks will automatically run when you commit to:
   - Format code with `cargo fmt`
   - Check code with `cargo clippy`
   - Fix trailing whitespace and file endings
4. You can also run the hooks manually: `pre-commit run --all-files`
5. Run tests: `cargo test`

## Commit Message Guidelines

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification for commit messages:

- `feat:` for new features
- `fix:` for bug fixes
- `docs:` for documentation changes
- `style:` for formatting changes
- `refactor:` for code refactoring
- `test:` for adding or updating tests
- `chore:` for maintenance tasks

Examples:
- `feat: add support for custom time formats`
- `fix: resolve authentication token expiration issue`
- `docs: update installation instructions`

## Pull Request Process

1. Update your fork with the latest changes from upstream:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```
2. Push your changes to your fork: `git push origin feature/your-feature-name`
3. Open a pull request against the main branch of the original repository
4. Fill out the pull request template with details about your changes
5. Wait for maintainers to review your pull request
6. Address any feedback or requested changes
7. Once approved, your pull request will be merged

## Release Process

Releases are managed automatically using go-semantic-release and GoReleaser with GitHub Actions. When changes are merged to the main branch, the following happens:

1. Tests and checks are run
2. go-semantic-release analyzes commit messages and creates a new version tag if needed
3. If a new tag was created, GoReleaser builds binaries for multiple platforms
4. A new release is created with automatically generated release notes and binary artifacts

The configuration for go-semantic-release is stored in the `.gsr.yaml` file and GoReleaser configuration is in `.goreleaser.yaml` at the root of the project.

## Testing

- Write tests for all new features and bug fixes
- Make sure all existing tests pass before submitting a pull request
- Run tests using `cargo test`

## Documentation

- Update the README.md file if your changes affect usage
- Add inline documentation for new functions and code
- Consider updating or adding examples if appropriate

## Getting Help

If you need help or have questions, you can:
- Open an issue on GitHub
- Reach out to the maintainers

Thank you for contributing to timedctl-rs!
