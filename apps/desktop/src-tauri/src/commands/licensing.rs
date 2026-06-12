use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseValidationResult {
    #[serde(rename = "valid")]
    pub valid: bool,
    pub tier: String,
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<String>,
    #[serde(rename = "messageCode")]
    pub message_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CloudShareResponse {
    pub url: String,
    #[serde(rename = "expiresAt")]
    pub expires_at: String,
}

#[tauri::command]
pub async fn validate_license_key(key: String) -> Result<LicenseValidationResult, String> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return Ok(LicenseValidationResult {
            valid: true,
            tier: "community".into(),
            expires_at: None,
            message_code: "noKey".into(),
        });
    }

    if trimmed.starts_with("BS-PRO-") {
        return Ok(LicenseValidationResult {
            valid: true,
            tier: "pro".into(),
            expires_at: None,
            message_code: "validPro".into(),
        });
    }

    if trimmed.starts_with("BS-CLOUD-") {
        return Ok(LicenseValidationResult {
            valid: true,
            tier: "cloud".into(),
            expires_at: None,
            message_code: "validCloud".into(),
        });
    }

    if trimmed.starts_with("BS-TEAM-") {
        return Ok(LicenseValidationResult {
            valid: true,
            tier: "team".into(),
            expires_at: None,
            message_code: "validTeam".into(),
        });
    }

    Ok(LicenseValidationResult {
        valid: false,
        tier: "community".into(),
        expires_at: None,
        message_code: "invalidKey".into(),
    })
}

#[tauri::command]
pub async fn upload_for_share(file_path: String) -> Result<CloudShareResponse, String> {
    let filename = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("capture.png");

    let expires_at = chrono::Utc::now() + chrono::Duration::hours(168);

    Ok(CloudShareResponse {
        url: format!(
            "https://share.betterscreenshoot.app/beta/{}",
            urlencoding::encode(filename)
        ),
        expires_at: expires_at.to_rfc3339(),
    })
}
