# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.16.0] - 2022-03-10

### Added

- `clean-retained` subcommand to clean retained topics.
- Interactive: Press Delete or Backspace to clean retained topics from the selected topic tree.
- Alias for log subcommand: `mqttui l`.

### Changed

- Interactive: Improve performance of the graphs.
- Interactive: Reimplement the mqtt history data structure to be both simpler and faster.

### Fixed

- Interactive: Don't plot non-finite numbers.
- Do not display mqtt password from env in --help.

## [0.15.0] - 2022-02-14

### Added

- New `log` subcommand to watch topics and prints them to stdout.

### Changed

- CLI: `ValueHint` improves autocompletion.

### Fixed

- Interactive: Don't error on quit about the main thread being gone.

## [0.14.0] - 2022-01-31

### Added

- Pass MQTT credentials via CLI.
- Allow environment variable arguments.

### Changed

- CLI: visible publish subcommand aliases.
- Performance improvements.

### Fixed

- Interactive: Show values on full width.
- Improve error messages.
