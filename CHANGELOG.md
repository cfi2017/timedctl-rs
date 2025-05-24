# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Enhanced interactive TUI with fuzzy search capabilities
- Refresh token support to minimize authentication prompts
- Support for start time when creating activities
- Stream Deck integration documentation
- GitHub Actions workflows for CI/CD and releases
- Non-interactive mode for all commands to support scripting

### Changed
- Extended token expiration buffer to 1 hour
- Improved error handling and validation
- Enhanced activity and report displays with better formatting
- Made interactive mode the default
- Duration input now rounds to 15-minute increments

### Fixed
- Task relationships display in reports
- Authentication flow with proper token refresh
- Activity selection and display

## [0.1.0] - 2023-12-01

### Added
- Initial release with basic functionality
- Command-line interface for time tracking
- Support for activities and reports
- OpenID Connect authentication
