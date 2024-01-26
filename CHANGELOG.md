# Changelog

All notable changes to the `dom_finder` crate will be documented in this file.

## [Unreleased]

## Changed
- `Finder` instance's lifetime now doesn't depends on `Config` lifetime.

## [0.2.3] - 2024-01-17

### Changed
- Change behavior of the `regex` procedure for `pipeline`. Now it captures all groups (excluding matching groups) from the first match.

### Added
- Add a new procedure for the `pipeline` -- `regex_find`. It returns the first entire match in the given string (result value).

## [0.2.2] - 2024-01-16

### Added
- Add a new sanitization policy for the `pipeline` -- `policy_common`. It is a combination of all previous policies.
- Add `dom_finder::Finder::parse_document`.

## [0.2.1] - 2024-01-14

### Changed
- Fix and extend the documentation.

## [0.2.0] - 2024-01-14

### Added
- Extend `Value` capabilities.
- Extend documentation.
- Add more test examples.

## [0.1.0] - 2024-01-10

### Added
- Initial release of the `dom_finder` crate.
