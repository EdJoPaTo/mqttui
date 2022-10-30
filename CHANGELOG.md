# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Vim paging keys

## [0.18.0] - 2022-10-06

### Changed

- Smaller Info Header at the top (only 2 lines instead of 5)
- Performance: Simplify interactive drawing logic

### Fixed

- Clean retained from interactive now uses the same mqtt connection. It now publishes on all topics below rather than only retained ones to ensure everything is being cleaned.
- Precompiled x86_64 build works again on Debian 11

## [0.17.0] - 2022-09-07

### Added

- Support TLS encryption (via `--broker mqtts://`)
- Support websockets (via `--broker ws://` or `--broker wss://`)
- Mouse clicks now select the overview / JSON Payload area
- Home/End key support for overview and JSON Payload area
- PageUp/Down key support for the overview
- Add key hints in the bottom of the TUI

### Changed

- Combine MQTT `--broker host` and `--port port` into single `--broker mqtt://host:port`
- Require URL scheme prefix for `--broker` (like `mqtt://`)
- Performance: Do not store topic on each history entry
- Performance: Store `String` as `Box<str>`
- Performance: Store less data on non-UTF8 payload
- Performance: Use RwLock over Mutex
- Performance: Simplify interactive drawing logic
- Performance: Simplify non-interactive output logic
- Performance: Only update TUI when key/mouse event did something

### Fixed

- Simplify JSON Payload view of non-Object/Array datatypes (don't prefix with "root: ")

## [0.16.2] - 2022-05-01

### Fixed

- Dont crash / endless loop on payloads bigger than 10 kB.

### Changed

- Parse payload content (JSON/UTF8-String/other) only once. Before it was done on every display update.
- Less data cloning while showing the graph improves performance.

## [0.16.1] - 2022-03-23

### Added

- Package as deb/rpm packages.

### Fixed

- Only panic on MQTT startup errors. Continue on errors when the startup worked fine.

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
