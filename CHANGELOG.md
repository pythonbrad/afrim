# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

### Added

### Fixed

## [0.7.0] - 2026-07-07

### Changed

- (lib) Updated dependencies. [(#295)](https://github.com/fodydev/afrim/pull/295)
- Removed the inhibit feature. [(#294)](https://github.com/fodydev/afrim/pull/294)
- (lib) Improved the afrim api. [(#286)](https://github.com/fodydev/afrim/pull/286) & [(#292)](https://github.com/fodydev/afrim/pull/292)
- Updated rust toolchain version to 1.96. [(#279)](https://github.com/fodydev/afrim/pull/279) & [(#280)](https://github.com/fodydev/afrim/pull/280)

### Added

- Added the possibility to adjust the prediction confidence in the configuration file. [(#291)](https://github.com/fodydev/afrim/pull/291)

### Fixed

- Improved the performance of configuration files loading. [(#281)](https://github.com/fodydev/afrim/pull/281)
- Improved the performance of the translation system. [(#278)](https://github.com/fodydev/afrim/pull/278)

## [0.6.2] - 2025-10-23

- (lib) Updated dependencies. [(#275)](https://github.com/fodydev/afrim/pull/275)

## [0.6.1] - 2025-06-12

### Changed

- (lib) Updated dependencies. [(#267)](https://github.com/fodydev/afrim/pull/267)

## [0.6.0] - 2024-04-28

### Added

- Resume an ended sequence. [(#211)](https://github.com/fodydev/afrim/pull/211) & [(#213)](https://github.com/fodydev/afrim/pull/213)

### Changed

- (lib) Improved the frontend API. [(#230)](https://github.com/fodydev/afrim/pull/230)
- (lib) Simplify the handling of predicates. [(#229)](https://github.com/fodydev/afrim/pull/229)
- (lib) Removed useless allocation. [(#212)](https://github.com/fodydev/afrim/pull/212) & [(#214)](https://github.com/fodydev/afrim/pull/214) & [(#217)](https://github.com/fodydev/afrim/pull/217) & [(#218)](https://github.com/fodydev/afrim/pull/218)
- (lib) Increased performance of the translator. [(#204)](https://github.com/fodydev/afrim/pull/204)
- (lib) Improve error handling. [(#203)](https://github.com/fodydev/afrim/pull/203)

### Fix
- (lib) Ignore unpaired data when loading the dataset. [(#215)](https://github.com/fodydev/afrim/pull/215)

## [0.5.4] - 2024-03-05

### Added

- (lib) Added the serialization/deserialization feature. [(#179)](https://github.com/fodydev/afrim/pull/179) & [(#181)](https://github.com/fodydev/afrim/pull/181)

### Changed

- (lib) Refactor to improve the code readability. [(#163)](https://github.com/fodydev/afrim/pull/163) & [(#176)](https://github.com/fodydev/afrim/pull/176)

### Fixed

- Issue with the overwritting of the autocapitalization. [(#183)](https://github.com/fodydev/afrim/pull/#183)
- (lib) Incomplete extension of the wasm support. [(#146)](https://github.com/fodydev/afrim/pull/146)

## [0.5.3] - 2024-01-31

### Added

- Update dependencies. [(#144)](https://github.com/fodydev/afrim/pull/144)
- (lib) Extension of the wasm support. [(#142)](https://github.com/fodydev/afrim/pull/142)

## [0.5.2] - 2023-11-11

### Fixed

- (lib) Improve the translation system. [(#127)](https://github.com/fodydev/afrim/pull/127)

## [0.5.1] - 2023-11-10

### Fixed

- Fixed autocompletion in inhibit mode. [(#122)](https://github.com/fodydev/afrim/pull/122)

### Added

- (lib) Support for wasm (limited to the core). [(#120)](https://github.com/fodydev/afrim/pull/120)

## [0.5.0] - 2023-10-24

### Added

- Execution of afrim in test only mode. [(#93)](https://github.com/fodydev/afrim/pull/93)
- Auto-correction. [(#102)](https://github.com/fodydev/afrim/pull/102)
- Full immersion mode for non-latin languages. [(#111)](https://github.com/fodydev/afrim/pull/111)

### Changed

- Change the project name . [(#112)](https://github.com/fodydev/afrim/pull/112)
- Make afrim more modular (service, config, memory, ...). [(#99)](https://github.com/fodydev/afrim/pull/99)

### Fixed

- Update afrim special keys. [(#104)](https://github.com/fodydev/afrim/pull/104)

## [0.4.0] - 2023-09-16

### Added

- Extension scripting via the Rhai scripting language. [(#68)](https://github.com/fodydev/afrim/pull/68)
- Predication system. [(#72)](https://github.com/fodydev/afrim/pull/72) & [(#75)](https://github.com/fodydev/afrim/pull/75)
- Added a proper way to verify when the cursor is empty in the library. [(#86)](https://github.com/fodydev/afrim/pull/86)

### Changed

- Split afrim into separate components (processor and translator). [(#72)](https://github.com/fodydev/afrim/pull/72)

### Fixed

- Restricted the auto_capitalize by configuration file. [(#79)](https://github.com/fodydev/afrim/pull/79)
- Improved sequence detection. [(#74)](https://github.com/fodydev/afrim/pull/74)
- Improved error handling and made it more understandable. [(#69)](https://github.com/fodydev/afrim/pull/69)

## [0.3.1] - 2023-08-13

### Added

- Implement the auto capitalization. [(#56)](https://github.com/fodydev/afrim/pull/56)

### Fixed

- Improve the pause/resume way via double pressing of CTRL key. [(#54)](https://github.com/fodydev/afrim/pull/54)

### Removed

- Drop function key F1-12 which was reserved for special purposes. [(#62)](https://github.com/fodydev/afrim/pull/62)

## [0.3.0] - 2023-06-02

### Added

- Reserved function key F1-12 for special purposes. [(#52)](https://github.com/fodydev/afrim/pull/52)
- Add a pause/resume way via double pressing of CTRL key [(#50)](https://github.com/fodydev/afrim/pull/50) & [(#49)](https://github.com/fodydev/afrim/pull/49)

### Fixed

- (lib) Problem of endless sequence [(#44)](https://github.com/fodydev/afrim/pull/44)
- Correct problem of excessive backspace [(#43)](https://github.com/fodydev/afrim/pull/43)
- The Capslock key don't reset the cursor [(#45)](https://github.com/fodydev/afrim/pull/45)

## [0.2.2] - 2023-05-17

### Fixed

- Correct logic of writing back of the previous out after backspace [(#39)](https://github.com/fodydev/afrim/pull/39)

## [0.2.1] - 2023-05-15

### Added

- Add a void frontend [(#33)](https://github.com/fodydev/afrim/pull/33)
- Implement a config file [(#31)](https://github.com/fodydev/afrim/pull/31)

### Fixed

- Replace echap key by pause key since the purpose of the pause key is not related to input field [(#30)](https://github.com/fodydev/afrim/pull/30)
- Fix problem of not human character input [(#29)](https://github.com/fodydev/afrim/pull/29)

## [0.2.0] - 2023-05-09

### Added

- Implement the initial binary application [(#8)](https://github.com/fodydev/afrim/pull/8)
- (lib) Implement a cursor [(#6)](https://github.com/fodydev/afrim/pull/6)
- (lib) Each node hold his key [(#16)](https://github.com/fodydev/afrim/pull/16)
- (lib) Each node hold his depth on the tree [(#5)](https://github.com/fodydev/afrim/pull/5)

### Changed

- (lib) Rename bst to text_buffer [(#9)](https://github.com/fodydev/afrim/pull/9)

## [0.1.1] - 2023-04-28

### Added

- (lib) Implement the initial library
