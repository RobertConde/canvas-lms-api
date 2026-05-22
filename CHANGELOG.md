# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.1] - 2026-05-22

### Fixed
- All integration test files now use `collect_all().await` instead of `futures::StreamExt::collect`
  so they compile correctly under `--no-default-features --features blocking` (the `futures` crate
  is not available in that CI configuration).
- `docs/RELEASING.md` expanded Step 4 to explicitly run all three CI matrix configurations
  locally before tagging.

## [0.5.0] - 2026-05-22

### Added
- **Instance methods across all core resources** — every major struct now carries a
  `requester` field and supports method calls directly on the returned object, matching
  the Python `canvasapi` library's interface:
  - **Tab** — `update`
  - **Enrollment** — `accept`, `reject`, `deactivate`, `reactivate`
  - **Progress** — `query`
  - **FeatureFlag** — `delete`, `set_feature_flag`
  - **File** — `update`, `delete`, `get_contents`, `download`
  - **Folder** — `update`, `delete`, `get_files`, `get_folders`, `create_folder`, `copy_file`
  - **Page + PageRevision** — `edit`, `delete`, `get_revisions`, `get_revision_by_id`,
    `show_latest_revision`, `revert_to_revision`; group context support
  - **Section** — `edit`, `delete`, `enroll_user`, `get_enrollments`,
    `cross_list_section`, `decross_list_section`, `get_assignment_override`,
    `get_multiple_submissions`, `submissions_bulk_update`
  - **Module** — `edit`, `delete`, `relock`, `get_module_items`, `get_module_item`,
    `create_module_item` (validates content_id requirement per type)
  - **ModuleItem** — `edit`, `delete`, `complete`, `uncomplete`
  - **DiscussionTopic** — `update`, `delete`, `post_entry`, `get_topic_entries`,
    `get_entries`, `mark_as_read`, `mark_as_unread`, `mark_entries_as_read`,
    `mark_entries_as_unread`, `subscribe`, `unsubscribe`; course and group context
  - **DiscussionEntry** — `update`, `delete`, `post_reply`, `get_replies`,
    `mark_as_read`, `mark_as_unread`, `rate` (validates 0–1 range)
  - **Submission** — `edit`, `mark_read`, `mark_unread`, `create_submission_peer_review`,
    `delete_submission_peer_review`, `get_submission_peer_reviews`
  - **Assignment** — `edit`, `delete`, `get_submissions`, `get_submission`, `submit`,
    `get_overrides`, `get_override`, `create_override`, `get_peer_reviews`,
    `get_gradeable_students`, `set_extensions`, `submissions_bulk_update`
  - **AssignmentGroup** — `edit`, `delete`
  - **AssignmentOverride** — `edit`, `delete`
  - **Quiz** — `edit`, `delete`, `create_question`, `get_question`, `get_questions`,
    `create_submission`, `get_submission`, `get_submissions`, `get_statistics`,
    `set_extensions`
  - **QuizSubmission** — `complete`, `get_submission_questions`, `get_times`,
    `update_score_and_comments`
  - **User** — `edit`, `get_profile`, `terminate_sessions`, `merge_into`,
    `get_avatars`, `get_page_views`, `get_observees`, `add_observee`, `remove_observee`,
    `show_observee`, `get_observers`, `create_pairing_code`, `get_colors`, `get_color`,
    `update_color`, `get_missing_submissions`, `get_files`, `get_folders`, `create_folder`,
    `get_file_quota`, `get_user_logins`, `get_settings`, `update_settings`,
    `create_communication_channel`, `get_authentication_events`, `get_features`,
    `get_enabled_features`, `export_content`, `get_content_exports`, `get_eportfolios`,
    `get_open_poll_sessions`, `get_closed_poll_sessions`
  - **Group** — `edit`, `delete`, `get_users`, `get_memberships`, `create_membership`,
    `get_membership`, `update_membership`, `remove_user`, `invite`, `get_files`,
    `get_file`, `get_folders`, `get_folder`, `create_folder`, `get_pages`, `get_page`,
    `create_page`, `get_discussion_topics`, `get_discussion_topic`,
    `create_discussion_topic`, `get_tabs`, `get_content_migrations`,
    `get_content_exports`, `preview_html`, `resolve_path`
  - **GroupMembership** — `update`, `remove_self`
  - **GroupCategory** — `update`, `delete`, `get_groups`, `get_users`,
    `create_group`, `assign_members`
- **New Course methods** — `conclude`, `reset`, `get_settings`, `update_settings`,
  `get_late_policy`, `get_multiple_submissions`, `submissions_bulk_update`,
  `enroll_user`, `create_module`, `create_discussion_topic`,
  `get_assignment_groups`, `create_assignment_group`,
  `get_group_categories`, `create_group_category`
- **New Account methods** — `update`, `get_subaccounts`, `create_subaccount`,
  `get_users`, `create_user`, `delete_user`, `get_courses`, `get_groups`,
  `get_group_categories`, `create_group_category`, `get_admins`, `create_admin`,
  `get_authentication_providers`, `get_reports`, `create_report`,
  `get_outcome_import_status`
- **`Requester::put_void`, `post_void_with_params`** — new helpers for endpoints
  that return 204 No Content
- **`Requester::get_url_bytes`** — raw URL GET for file content download

### Changed
- `Course::get_modules`, `get_module`, `get_assignments`, `get_assignment`,
  `create_assignment`, `get_quizzes`, `get_quiz`, `create_quiz`,
  `get_discussion_topics`, `get_discussion_topic`, `get_pages`, `get_page`,
  `get_tabs`, `get_enrollments`, `get_files` and other listing/fetching methods
  now inject the `requester` (and parent context ids where needed) into returned
  structs so instance methods work out of the box.
- `DiscussionTopic.course_id_ctx` (skip-serialized) added alongside the real
  `course_id` JSON field to carry the context course id for topics fetched through
  the Course API.

## [0.4.0] - 2026-05-20

### Added
- **Polls** — `Poll`, `PollChoice`, `PollSession`, `PollSubmission` structs with full CRUD
  matching the Python `canvasapi` surface. Client-level: `Canvas::get_poll`,
  `get_polls`, `create_poll`. Instance methods: `Poll::update`, `delete`,
  `get_choice`, `get_choices`, `create_choice`, `get_session`, `get_sessions`,
  `create_session`; `PollChoice::update`, `delete`; `PollSession::update`,
  `delete`, `open`, `close`, `get_submission`, `create_submission`.
- **Collaborations** — `Collaboration` + `Collaborator` structs.
  `Course::get_collaborations`, `Group::get_collaborations` (list only — no
  create/get-single endpoint exists in the Canvas API).
  `Collaboration::get_collaborators` (`GET /collaborations/:id/members`).
- **LTI Resource Links** — `LtiResourceLink` + `CreateLtiResourceLinkParams`.
  `Course::get_lti_resource_links`, `Course::get_lti_resource_link`,
  `Course::create_lti_resource_link`.
- **`impl futures::Stream for PageStream<T>`** — `PageStream<T>` now directly
  implements `futures::Stream` (gated on the `async` feature), enabling
  `StreamExt` methods (`next()`, `map()`, `filter()`, `collect()`, etc.)
  without any adapter. `collect_all()` remains available as a convenience.
- **`#[derive(CanvasResource)]` proc-macro** — new `canvas-lms-api-derive`
  workspace crate; generates the `fn req()` accessor on any resource struct
  carrying a `requester: Option<Arc<Requester>>` field. Applied to all 18
  existing resource structs and all new v0.4.0 structs.
- `Requester::delete_void` — DELETE helper for 204 No Content responses
  (used by poll deletes).
- `Group` promoted to a requester-bearing resource struct, enabling
  instance-level methods.

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

[0.5.1]: https://github.com/RobertConde/canvas-lms-api/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/RobertConde/canvas-lms-api/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/RobertConde/canvas-lms-api/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/RobertConde/canvas-lms-api/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/RobertConde/canvas-lms-api/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/RobertConde/canvas-lms-api/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/RobertConde/canvas-lms-api/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/RobertConde/canvas-lms-api/releases/tag/v0.1.0
