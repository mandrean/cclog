# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.12.0](https://github.com/mandrean/cclog/releases/tag/cclog-v0.12.0) - 2026-03-17

### Fixed

- collapse nested if-let to satisfy clippy collapsible_if
- harden link_style and sectionmap, add tests
- correct minor bugs and cleanup across lib and cli

### Other

- bump MSRV to 1.88.0 (required by time 0.3.47)
- replace hand-rolled JSON with serde_json, add writer tests
- modernize clog.rs with helpers, better API, and tests
- restructure as cclog workspace monorepo
