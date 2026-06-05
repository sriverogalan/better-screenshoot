use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseValidationResult {
    pub valid: bool,
    pub tier: String,
    pub expires_at: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CloudShareResponse {
    pub url: String,
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
            message: "Sin clave — modo Community".into(),
        });
    }

    if trimmed.starts_with("BS-PRO-") {
        return Ok(LicenseValidationResult {
            valid: true,
            tier: "pro".into(),
            expires_at: None,
            message: "Licencia Pro válida".into(),
        });
    }

    if trimmed.starts_with("BS-CLOUD-") {
        return Ok(LicenseValidationResult {
            valid: true,
            tier: "cloud".into(),
            expires_at: None,
            message: "Licencia Cloud válida".into(),
        });
    }

    if trimmed.starts_with("BS-TEAM-") {
        return Ok(LicenseValidationResult {
            valid: true,
            tier: "team".into(),
            expires_at: None,
            message: "Licencia Team válida".into(),
        });
    }

    Ok(LicenseValidationResult {
        valid: false,
        tier: "community".into(),
        expires_at: None,
        message: "Clave de licencia no válida".into(),
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
