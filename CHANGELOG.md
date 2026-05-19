# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2026-05-19

### Fixed
- README: corrected license from AGPLv3 to MIT
- Moved DESIGN.md and RELEASING.md into `docs/`

## [0.1.0] - 2026-05-19

### Added
- `Canvas` client with async methods for courses, users, and accounts
- `CanvasBlocking` synchronous wrapper (feature = `blocking`)
- `PageStream<T>` — lazy paginated stream driven by Canvas `Link` headers
- `CanvasError` enum mapping all Canvas HTTP error codes (400–429)
- Phase 1 resource structs: Course, User, Assignment, Submission, Enrollment,
  Section, Module, Quiz, Group, Account, File, Folder, Page, DiscussionTopic,
  Progress, Tab
- Course sub-methods: get/create assignment, quiz, module, page, discussion
  topic, file upload, tabs, groups, sections, enrollments, users
- User sub-methods: get courses, get enrollments
- Two-step Canvas file upload (`course.upload_file`) with `while(1);` stripping
- Bracket-notation parameter serialization (`params::wrap_params`)
- Typed parameter builder structs for courses, assignments, quizzes, and users
- Async HTTP layer via `reqwest` with form-encoded POST/PUT/PATCH bodies
- CI: fmt, clippy, tests, doc build, MSRV 1.75 check
- MIT license

[0.1.0]: https://github.com/RobertConde/canvas-lms-api/releases/tag/v0.1.0
