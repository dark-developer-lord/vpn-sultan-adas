use axum::{
    extract::{multipart::Multipart, Path},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tokio::io::AsyncReadExt;
use sha2::{Sha256, Digest};

// ==================== TYPES ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct FileUploadResponse {
    pub file_id: String,
    pub filename: String,
    pub url: String,
    pub size: u64,
    pub mime_type: String,
    pub uploaded_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfilePictureResponse {
    pub user_id: String,
    pub picture_url: String,
    pub uploaded_at: String,
}

// ==================== CONFIGURATION ====================

pub struct FileUploadConfig {
    /// Max file size (default: 5MB)
    pub max_file_size: u64,
    /// Max profile picture size (default: 2MB)
    pub max_profile_pic_size: u64,
    /// Allowed MIME types for profile pictures
    pub allowed_image_types: Vec<&'static str>,
    /// Local storage path
    pub storage_path: String,
    /// S3 bucket (optional)
    pub s3_bucket: Option<String>,
}

impl Default for FileUploadConfig {
    fn default() -> Self {
        Self {
            max_file_size: 5 * 1024 * 1024,      // 5MB
            max_profile_pic_size: 2 * 1024 * 1024, // 2MB
            allowed_image_types: vec!["image/jpeg", "image/png", "image/webp"],
            storage_path: "./uploads".to_string(),
            s3_bucket: None,
        }
    }
}

// ==================== FILE UPLOAD SERVICE ====================

pub struct FileUploadService {
    config: FileUploadConfig,
    db: Database,
}

impl FileUploadService {
    pub fn new(config: FileUploadConfig, db: Database) -> Self {
        Self { config, db }
    }

    /// Upload profile picture
    pub async fn upload_profile_picture(
        &self,
        user_id: &str,
        mut multipart: Multipart,
    ) -> Result<ProfilePictureResponse> {
        let mut field = multipart
            .next_field()
            .await?
            .ok_or(AppError::BadRequest("No file provided".to_string()))?;

        let mime_type = field
            .content_type()
            .map(|m| m.to_string())
            .unwrap_or_default();

        // Validate MIME type
        if !self.config.allowed_image_types.contains(mime_type.as_str()) {
            return Err(AppError::BadRequest(format!(
                "Invalid file type: {}",
                mime_type
            )));
        }

        // Read file data
        let mut data = Vec::new();
        field.read_to_end(&mut data).await?;

        // Validate file size
        if data.len() as u64 > self.config.max_profile_pic_size {
            return Err(AppError::BadRequest(
                "Profile picture too large".to_string(),
            ));
        }

        // Generate filename with hash
        let hash = self.calculate_hash(&data);
        let file_ext = self.get_file_extension(&mime_type);
        let filename = format!("profile-{}-{}.{}", user_id, hash, file_ext);

        // Save file
        let file_path = format!("{}/{}", self.config.storage_path, filename);
        self.save_file(&file_path, &data).await?;

        // Generate URL
        let url = format!("/api/uploads/{}", filename);

        // Update database
        self.db
            .execute(
                "UPDATE users SET picture_url = $1, picture_uploaded_at = NOW() WHERE id = $2",
                &[&url, &user_id],
            )
            .await?;

        // Log activity
        log_audit_event(
            user_id,
            "PROFILE_PICTURE_UPLOADED",
            &format!("Profile picture updated: {}", filename),
            &self.db,
        )
        .await?;

        Ok(ProfilePictureResponse {
            user_id: user_id.to_string(),
            picture_url: url,
            uploaded_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Upload general file (with virus scan)
    pub async fn upload_file(
        &self,
        user_id: &str,
        mut multipart: Multipart,
    ) -> Result<FileUploadResponse> {
        let mut field = multipart
            .next_field()
            .await?
            .ok_or(AppError::BadRequest("No file provided".to_string()))?;

        let filename = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "upload".to_string());

        let mime_type = field
            .content_type()
            .map(|m| m.to_string())
            .unwrap_or_default();

        // Read file data
        let mut data = Vec::new();
        field.read_to_end(&mut data).await?;

        // Validate file size
        if data.len() as u64 > self.config.max_file_size {
            return Err(AppError::BadRequest("File too large".to_string()));
        }

        // Scan for malware (integrate with ClamAV or similar)
        self.scan_for_malware(&data).await?;

        // Generate safe filename
        let file_id = Uuid::new_v4().to_string();
        let hash = self.calculate_hash(&data);
        let safe_filename = format!("{}-{}", file_id, sanitize_filename(&filename));

        // Save file
        let file_path = format!("{}/{}", self.config.storage_path, safe_filename);
        self.save_file(&file_path, &data).await?;

        // Generate URL
        let url = format!("/api/uploads/{}", safe_filename);

        // Store metadata in database
        self.db
            .execute(
                "INSERT INTO file_uploads (file_id, user_id, original_filename, storage_filename, mime_type, size, hash, url, uploaded_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())",
                &[
                    &file_id,
                    user_id,
                    &filename,
                    &safe_filename,
                    &mime_type,
                    &(data.len() as i64).to_string(),
                    &hash,
                    &url,
                ],
            )
            .await?;

        log_audit_event(
            user_id,
            "FILE_UPLOADED",
            &format!("File uploaded: {} ({} bytes)", filename, data.len()),
            &self.db,
        )
        .await?;

        Ok(FileUploadResponse {
            file_id,
            filename,
            url,
            size: data.len() as u64,
            mime_type,
            uploaded_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Delete file
    pub async fn delete_file(&self, user_id: &str, file_id: &str) -> Result<()> {
        // Verify ownership
        let file = self
            .db
            .query_one(
                "SELECT storage_filename FROM file_uploads WHERE file_id = $1 AND user_id = $2",
                &[&file_id, user_id],
            )
            .await?
            .ok_or(AppError::NotFound("File not found".to_string()))?;

        let storage_filename: String = file.get(0);

        // Delete from filesystem
        let file_path = format!("{}/{}", self.config.storage_path, storage_filename);
        tokio::fs::remove_file(&file_path).await.ok();

        // Delete from database
        self.db
            .execute(
                "DELETE FROM file_uploads WHERE file_id = $1 AND user_id = $2",
                &[&file_id, user_id],
            )
            .await?;

        log_audit_event(
            user_id,
            "FILE_DELETED",
            &format!("File deleted: {}", file_id),
            &self.db,
        )
        .await?;

        Ok(())
    }

    // ==================== HELPERS ====================

    fn calculate_hash(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())[..16].to_string()
    }

    fn get_file_extension(&self, mime_type: &str) -> &'static str {
        match mime_type {
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/webp" => "webp",
            _ => "bin",
        }
    }

    async fn save_file(&self, path: &str, data: &[u8]) -> Result<()> {
        // Create directory if it doesn't exist
        let dir = std::path::Path::new(path).parent();
        if let Some(dir) = dir {
            tokio::fs::create_dir_all(dir).await?;
        }

        // Write file
        tokio::fs::write(path, data).await?;
        Ok(())
    }

    async fn scan_for_malware(&self, data: &[u8]) -> Result<()> {
        // TODO: Integrate with ClamAV or VirusTotal API
        // For now, just check magic bytes
        if is_suspicious_file(data) {
            return Err(AppError::BadRequest(
                "File appears to be malicious".to_string(),
            ));
        }
        Ok(())
    }
}

// ==================== HANDLERS ====================

/// Upload profile picture
#[post("/users/me/picture")]
pub async fn upload_profile_picture(
    auth: AuthLayer,
    db: Database,
    multipart: Multipart,
) -> Result<Json<ProfilePictureResponse>> {
    let config = FileUploadConfig::default();
    let service = FileUploadService::new(config, db);
    let response = service.upload_profile_picture(&auth.user_id, multipart).await?;

    Ok(Json(response))
}

/// Upload general file
#[post("/files/upload")]
pub async fn upload_file(
    auth: AuthLayer,
    db: Database,
    multipart: Multipart,
) -> Result<Json<FileUploadResponse>> {
    let config = FileUploadConfig::default();
    let service = FileUploadService::new(config, db);
    let response = service.upload_file(&auth.user_id, multipart).await?;

    Ok(Json(response))
}

/// Delete file
#[delete("/files/:file_id")]
pub async fn delete_file(
    auth: AuthLayer,
    Path(file_id): Path<String>,
    db: Database,
) -> Result<StatusCode> {
    let config = FileUploadConfig::default();
    let service = FileUploadService::new(config, db);
    service.delete_file(&auth.user_id, &file_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

// ==================== DATABASE SCHEMA ====================

pub const FILE_UPLOADS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS file_uploads (
    file_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    original_filename TEXT NOT NULL,
    storage_filename TEXT NOT NULL UNIQUE,
    mime_type TEXT NOT NULL,
    size BIGINT NOT NULL,
    hash TEXT NOT NULL,
    url TEXT NOT NULL,
    uploaded_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE
);

-- Add picture columns to users table if not exists
ALTER TABLE users ADD COLUMN IF NOT EXISTS picture_url TEXT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS picture_uploaded_at TIMESTAMP WITH TIME ZONE;

CREATE INDEX IF NOT EXISTS idx_file_uploads_user_id ON file_uploads(user_id);
CREATE INDEX IF NOT EXISTS idx_file_uploads_uploaded_at ON file_uploads(uploaded_at);
"#;

// ==================== SECURITY HELPERS ====================

fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect()
}

fn is_suspicious_file(data: &[u8]) -> bool {
    // Check for suspicious magic bytes
    let suspicious_magics = [
        b"MZ",           // Windows executable
        b"PK",           // ZIP
        b"\x1f\x8b",     // GZIP
        b"BZh",          // BZIP
    ];

    suspicious_magics.iter().any(|magic| data.starts_with(magic))
}
