# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial scaffold: `Canvas` client, `PageStream<T>` pagination, `CanvasError` enum
- Phase 1 resource structs: Course, User, Assignment, Submission, Enrollment, Section,
  Module, Quiz, Group, Account, File, Folder, Page, DiscussionTopic, Progress, Tab
- Bracket-notation parameter serialization (`params::wrap_params`)
- Async HTTP layer via `reqwest`
- CI workflows (fmt, clippy, test, doc, MSRV 1.75)
- AGPLv3 license
