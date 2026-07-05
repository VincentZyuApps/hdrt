use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HdrtWarning {
    pub code: String,
    pub message: String,
    pub hint: Option<String>,
}

impl HdrtWarning {
    pub fn with_hint(
        code: impl Into<String>,
        message: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            hint: Some(hint.into()),
        }
    }
}
