# Changelog

All notable changes to the `dom_finder` crate will be documented in this file.

## [0.3.2] - 2024-10-24

### Changed
- Update `dom_query`'s version to `0.6.0`.
- Minor code changed due to `dom_query::Selection` doesn't require `&mut` anymore.

## [0.3.1]

### Added
- Start using `Selection::select_single_matcher` if there is not corresponding `many` option.

### Changed
- Update `dom_query`'s version to `0.5.0`.

## [0.3.0] - 2024-10-07

### Changed
- Update `dom_query`'s version to `0.4.2`.

### Added
- Now `Config.extract` also supports `inner_html` value, which allows to extract html without the element's tag (only children).

## [0.2.7] - 2024-10-01

### Changed
- Update `dom_query`'s version to `0.4.0`.
- Update other dependencies.

## [0.2.6] - 2024-04-04

### Changed
- Update `dom_query`'s version to `0.3.5`.

## [0.2.5] - 2024-02-17

### Changed
- Update `dom_query`'s version to `0.3.4`.

### Added
- implement `TryFrom<Config>` trait for `Finder`.

## [0.2.4] - 2024-01-27

### Changed
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
