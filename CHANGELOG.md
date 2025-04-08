# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a change log](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.22.1] - 2025-04-08

### Added

- Honors environment variable SSLKEYLOGFILE to dump TLS encryption secrets.

### Changed

- Interactive: Improve render performance of the history table from O(entries) to O(height)

### Fixed

- Interactive: Scrolling behaviour of the history table

## [0.22.0] - 2025-03-01

### Added

- Interactive: `o` on the topic tree opens all currently known topics. `O` (Shift + `o`) closes all topics.
- Interactive: Remove history entries in the history table with `Del` or `Backspace`. This might be neat to cleanup some graphs while testing devices without restarting mqttui.
  As this changes the cached history and might be weird to understand (sending an empty payload to cleanup some retained values is something different) this is a somewhat hidden feature and isnt shown in the footer / key hints.

### Fixed

- Interactive: Only handle key pressed events and ignore released events.
- Dont fail when some platform certificates can't be loaded. Print a warning and continue. The needed certificates might be included that way.

## [0.21.1] - 2024-07-30

### Changed

- Interactive: Show cursor on search input

### Fixed

- Update to ratatui 0.26.3 to prevent panic at truncation of Unicode multi-width characters. See <https://github.com/ratatui-org/ratatui/pull/1089> for more details.

## [0.21.0] - 2024-04-17

### Changed

- Provide better error output on initial MQTT connection errors
- Interactive: When payload is focused it can occupy more space when needed.
- Interactive: Show total amount of messages in the topic overview title
- Log: Provide machine-readable newline-delimited output with `--json`
- Log: Print `--verbose` to stderr instead of stdout

## [0.20.0] - 2024-02-26

### Added

- Interactive: Topic search
- Interactive: History table entry is selectable (keyboard & mouse) to view a payload in detail
- Interactive: Scrolling moves view not selection and has scrollbars
- Interactive: graph plots values with units will ignore everything after the whitespace (`20.0 °C` → `20.0`)
- Publish from stdin (and with that from file contents)
- Support for decoding [MessagePack](https://msgpack.org/) payloads
- Support for binary payloads
- Interactive: Truncate payloads to ´--payload-size-limit´ for reduced RAM usage
- Auto-generated man pages from CLI definition (clap)

### Changed

- Interactive: Fewer borders for more content characters like longer topics in the overview
- Interactive: Display version & broker in the lower right corner
- Interactive: Display MQTT connection error in its own area
- Interactive: Only display keys in footer when useful
- Interactive: JSON Payload takes only required space for bigger history/graph view
- CLI: Group MQTT connection related options in --help
- Read One: Output raw payload or --pretty
- Build: always build with TLS support
- Performance: Debounce input events on interactive draw (especially noticeable on many events like scrolling)
- Performance: Fewer clones on interactive draw
- Performance: Don't keep Timezone information of each message
- Performance: Don't clone each incoming MQTT payload
- Performance: Don't clone TLS private key on startup

### Fixed

- Keep selected JSON object keys selected (by key, not by index as before)
- Always quit on `Ctrl` + `C` (`q` still only quits when not typing anything)
- Catch panics on interactive and clean up terminal correctly before displaying them

## [0.19.0] - 2023-05-17

### Added

- TLS client certificate authentication
- Interactive: Vim paging keys
- Interactive: Show messages per second instead of every n seconds when >1 per second
- Interactive: Allow subscribing to multiple topics
- Read One: New sub-command to receive one payload from a given topic

### Changed

- Performance: drop mutex locks faster
- Performance: less variable clones

## [0.18.0] - 2022-10-06

### Changed

- Smaller Info Header at the top (only 2 lines instead of 5)
- Performance: Simplify interactive drawing logic

### Fixed

- Clean retained from interactive now uses the same MQTT connection. It now publishes on all topics below rather than only retained ones to ensure everything is being cleaned.
- Precompiled x86_64 build works again on Debian 11

## [0.17.0] - 2022-09-07

### Added

- Support TLS encryption (via `--broker mqtts://`)
- Support web sockets (via `--broker ws://` or `--broker wss://`)
- Mouse clicks now select the overview / JSON Payload area
- Home/End key support for overview and JSON Payload area
- Page Up/Down key support for the overview
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

- Simplify JSON Payload view of non-Object/Array data types (don't prefix with "root: ")

## [0.16.2] - 2022-05-01

### Fixed

- Don't crash / endless loop on payloads bigger than 10 kB.

### Changed

- Parse payload content (JSON/UTF8-String/other) only once. Before it was done on every display update.
- Fewer data clones while showing the graph improves performance.

## [0.16.1] - 2022-03-23

### Added

- Package as deb/rpm packages.

### Fixed

- Only panic on MQTT startup errors. Continue on errors when the startup worked fine.

## [0.16.0] - 2022-03-10

### Added

- `clean-retained` sub-command to clean retained topics.
- Interactive: Press Delete or Backspace to clean retained topics from the selected topic tree.
- Alias for log sub-command: `mqttui l`.

### Changed

- Interactive: Improve performance of the graphs.
- Interactive: Reimplement the MQTT history data structure to be both simpler and faster.

### Fixed

- Interactive: Don't plot non-finite numbers.
- Do not display MQTT password from env in --help.

## [0.15.0] - 2022-02-14

### Added

- New `log` sub-command to watch topics and prints them to stdout.

### Changed

- CLI: `ValueHint` improves autocompletion.

### Fixed

- Interactive: Don't error on quit about the main thread being gone.

## [0.14.0] - 2022-01-31

### Added

- Pass MQTT credentials via CLI.
- Allow environment variable arguments.

### Changed

- CLI: visible publish sub-command aliases.
- Performance improvements.

### Fixed

- Interactive: Show values on full width.
- Improve error messages.
