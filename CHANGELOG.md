# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] - 2024-09-08

This release follows the [Dahlia Specification v1.0.0][spec].

### Added

- Builder pattern for the `Dahlia` struct
- Automatic color depth detection (builder has a method `with_auto_depth`)
- [Style-specific reset codes][spec-reset]
- [The `&_` escape code][spec-esc] and `escape` method
- The `clean_ansi` function should now handle way more ANSI escape codes
- Comprehensive test suite
- String conversions are now approximately ~20% faster

### Changed

- The "Blink" style code was changed from `&p` to `&k`
- The custom color syntax was changed from `&[#ffaff3]` to `&#ffaff3;` and now
  supports shorthand 3-digit codes
- The full reset code is now `&R` instead of `&r`
- The "Hide" style code was changed from `&k` to `&h`
- The `no_reset` parameter was renamed to `auto_reset` and now
  defaults to `True`

### Removed

- `Dahlia::reset`
- `Dahlia::test`
- Dahlia's `no_color` parameter
- The `&g` code

[spec]: https://github.com/dahlia-lib/spec/
[spec-reset]: https://github.com/dahlia-lib/spec/blob/main/SPECIFICATION.md#resetting
[spec-esc]: https://github.com/dahlia-lib/spec/blob/main/SPECIFICATION.md#escaping

## [1.1.0] - 2022-11-24

### Added

- Custom markers

## [1.0.0] - 2022-10-31

Initial release 🚀

[1.0.0]: https://github.com/trag1c/Dahlia.rs/releases/tag/1.0.0
[1.1.0]: https://github.com/trag1c/Dahlia.rs/compare/1.0.0...1.1.0
[2.0.0]: https://github.com/trag1c/Dahlia.rs/compare/1.1.0...2.0.0
