use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppErrorPayload {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<HashMap<String, serde_json::Value>>,
}

impl AppErrorPayload {
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            detail: None,
        }
    }

    pub fn with_detail(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.detail
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }

    pub fn to_emit_value(&self) -> Self {
        self.clone()
    }

    pub fn to_invoke_error(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| self.code.clone())
    }
}

pub fn app_error(code: &str) -> String {
    AppErrorPayload::new(code).to_invoke_error()
}

pub fn parse_invoke_error(error: &str) -> AppErrorPayload {
    serde_json::from_str::<AppErrorPayload>(error)
        .unwrap_or_else(|_| AppErrorPayload::new("captureFailed"))
}
