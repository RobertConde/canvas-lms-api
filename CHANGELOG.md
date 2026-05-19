# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-05-20

### Added
- **AppointmentGroup**: `Canvas::get_appointment_group`, `get_appointment_groups`,
  `create_appointment_group`; `AppointmentGroup::delete`, `edit`
- **CalendarEvent**: `Canvas::get_calendar_event`, `get_calendar_events`,
  `create_calendar_event`; `CalendarEvent::delete`, `edit`
- **Conversation**: `Canvas::get_conversation`, `get_conversations`,
  `create_conversation`; `Conversation::add_message`, `add_recipients`,
  `delete`, `delete_messages`, `edit`, `set_workflow_state`
- **EnrollmentTerm**: `Account::get_enrollment_term`, `get_enrollment_terms`,
  `create_enrollment_term`; `EnrollmentTerm::delete`, `edit`
- **EPortfolio / EPortfolioPage**: `Canvas::get_eportfolio`;
  `EPortfolio::delete`, `get_pages`, `moderate`, `restore`
- **GradingPeriod**: `Course::get_grading_periods`;
  `GradingPeriod::update`, `delete`
- **GradingStandard**: `Account::get_grading_standards`, `get_grading_standard`,
  `create_grading_standard`; `Course::get_grading_standards`, `create_grading_standard`
- **JWT**: `Canvas::create_jwt`, `refresh_jwt`; `CanvasJwt` struct
- **ContentExport**: `Account::get_content_export`, `get_content_exports`,
  `create_content_export`; `Course::get_content_export`, `get_content_exports`,
  `create_content_export`; `ContentExportParams` builder
- **GradeChangeLog**: `Course::get_grade_change_events` returning
  `PageStream<GradeChangeEvent>`; handles Canvas `{"events":[...]}` wrapper
- **Feature / FeatureFlag**: `Account::get_features`, `get_feature_flag`,
  `get_enabled_features`; `Course::get_features`, `get_feature_flag`,
  `get_enabled_features`
- **PlannerNote / PlannerOverride**: `Canvas::get_planner_note`,
  `get_planner_notes`, `create_planner_note`; `Canvas::get_planner_override`,
  `get_planner_overrides`, `create_planner_override`; full CRUD on each struct
- **Role**: `Account::get_role`, `get_roles`, `create_role`, `deactivate_role`,
  `activate_role`, `update_role`
- **Convenience client methods**: `Canvas::get_section`, `get_group`, `get_file`,
  `get_folder`, `get_progress`, `get_outcome`

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

[0.3.0]: https://github.com/RobertConde/canvas-lms-api/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/RobertConde/canvas-lms-api/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/RobertConde/canvas-lms-api/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/RobertConde/canvas-lms-api/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/RobertConde/canvas-lms-api/releases/tag/v0.1.0
