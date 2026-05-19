use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowState {
    Active,
    Available,
    Completed,
    Created,
    Deleted,
    Exported,
    Imported,
    Importing,
    Invited,
    Processed,
    Queued,
    Running,
    Succeeded,
    Aborted,
    Published,
    Unpublished,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EnrollmentType {
    StudentEnrollment,
    TeacherEnrollment,
    TaEnrollment,
    DesignerEnrollment,
    ObserverEnrollment,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionType {
    OnlineTextEntry,
    OnlineUrl,
    OnlineUpload,
    MediaRecording,
    StudentAnnotation,
    OnlineQuiz,
    DiscussionTopic,
    ExternalTool,
    NotGraded,
    #[serde(other)]
    Unknown,
}
