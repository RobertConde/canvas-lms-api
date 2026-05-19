// Typed parameter builder structs for each Canvas API resource.
// Each struct derives Serialize + Default so it can be passed to
// flatten_params() for query-string construction.
pub mod assignment_params;
pub mod course_params;
