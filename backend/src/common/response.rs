use serde::Serialize;

#[derive(Serialize)]
pub struct GenericResponse<T> {
    pub data: T,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}