# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-05-19

### Added
- **New Quizzes feature** (`features = ["new-quizzes"]`): `NewQuiz` resource struct and
  `NewQuizParams` builder; `Course::get_new_quiz`, `get_new_quizzes`, `create_new_quiz`;
  `NewQuiz::update`, `delete`, `set_accommodations`; paginated stream via new
  `PageStream::new_with_injector_nq` (uses `/api/quiz/v1/` base URL)
- **GraphQL feature** (`features = ["graphql"]`): `GraphQL` client struct with `query()`
  method; `Canvas::graphql()` accessor; backed by `Requester::graphql_query()` hitting
  `/api/graphql` with JSON body
- **Account resource methods**: `get_account_calendar`, `get_all_account_calendars`,
  `get_external_tool`, `get_external_tools`, `create_external_tool`,
  `get_rubric`, `get_rubrics`, `create_rubric`, `get_outcome_group`,
  `get_outcome_group_links`, `create_outcome_group`, `get_content_migration`,
  `get_content_migrations`, `create_content_migration`, `get_migrators`,
  `get_sis_imports`, `get_sis_import`, `abort_sis_imports_pending`
- **Course resource methods**: external tools (get/list/create), rubrics (get/list/create),
  blueprint (get template, get subscriptions), content migrations (get/list/create/migrators),
  outcome groups (get/list/create), gradebook history (dates, details, submission history,
  uncollated submissions)
- **User resource methods**: `get_communication_channels`, `create_communication_channel`
- **New resource types**: `AccountCalendar`, `Blueprint` (template, migration, subscription,
  change record), `CommunicationChannel`, `ContentMigration` (migration issue, migrator),
  `ExternalTool`, `GradebookHistory` (day, grader, submission version, submission history),
  `NewQuiz`, `Outcome` (outcome group, outcome link), `Rubric` (rubric assessment, association),
  `SisImport`
- `Canvas::get_account` and `Canvas::get_accounts` now inject requester into `Account`
  struct so all account methods are available

### Changed
- MSRV bumped to 1.86 (required by transitive `icu_*` / `idna_adapter` deps)

## [0.1.2] - 2026-05-19

### Fixed
- README badges: correct CI URL casing, switch docs badge to shields.io

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

[0.2.0]: https://github.com/RobertConde/canvas-lms-api/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/RobertConde/canvas-lms-api/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/RobertConde/canvas-lms-api/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/RobertConde/canvas-lms-api/releases/tag/v0.1.0
