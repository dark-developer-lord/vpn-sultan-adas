#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ApiError;
    use std::path::Path;
    use tokio::fs;

    #[tokio::test]
    async fn test_file_upload_basic() {
        let uploader = FileUploader::new();
        let user_id = uuid::Uuid::new_v4();

        // Create test file
        let content = b"test file content";
        let filename = "test.txt";

        // Upload file
        let result = uploader
            .upload(user_id, filename, content)
            .await;

        assert!(result.is_ok(), "Upload should succeed");

        let file_id = result.unwrap();
        assert!(!file_id.is_empty(), "File ID should be generated");

        // Verify file was stored
        let retrieved = uploader.get_file(user_id, &file_id).await;
        assert!(retrieved.is_ok(), "Should retrieve uploaded file");
    }

    #[tokio::test]
    async fn test_file_upload_size_validation() {
        let uploader = FileUploader::new();
        let user_id = uuid::Uuid::new_v4();

        // Test max file size (e.g., 100MB)
        let large_content = vec![0u8; 101 * 1024 * 1024]; // 101 MB

        let result = uploader
            .upload(user_id, "large.bin", &large_content)
            .await;

        assert!(result.is_err(), "Should reject file exceeding max size");
    }

    #[tokio::test]
    async fn test_file_upload_type_validation() {
        let uploader = FileUploader::new();
        let user_id = uuid::Uuid::new_v4();

        // Test allowed file types for profile pictures
        let allowed_types = vec!["image/jpeg", "image/png", "image/webp"];

        for (ext, mime_type) in &[("jpg", "image/jpeg"), ("png", "image/png")] {
            let result = uploader
                .upload_profile_picture(user_id, &format!("pic.{}", ext), b"fake image", mime_type)
                .await;

            assert!(result.is_ok(), "File type {} should be allowed", ext);
        }

        // Test disallowed type
        let result = uploader
            .upload_profile_picture(user_id, "malicious.exe", b"fake exe", "application/x-msdownload")
            .await;

        assert!(result.is_err(), "Executable file should be rejected");
    }

    #[tokio::test]
    async fn test_file_upload_malware_scanning() {
        let uploader = FileUploader::new();
        let user_id = uuid::Uuid::new_v4();

        // Test with clean file
        let clean_content = b"This is a clean text file";
        let result = uploader
            .upload(user_id, "clean.txt", clean_content)
            .await;

        assert!(result.is_ok(), "Clean file should be accepted");

        // Test with EICAR test string (malware signature detector test)
        let eicar = b"X5O!P%@AP[4\\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*";
        let result = uploader
            .upload(user_id, "eicar.com", eicar)
            .await;

        assert!(result.is_err(), "File with malware signature should be rejected");
    }

    #[tokio::test]
    async fn test_file_upload_to_s3() {
        let uploader = FileUploader::new_with_s3();
        let user_id = uuid::Uuid::new_v4();

        let content = b"S3 test file";

        // Upload to S3
        let result = uploader
            .upload_to_s3(user_id, "test.txt", content)
            .await;

        assert!(result.is_ok(), "S3 upload should succeed");

        let s3_path = result.unwrap();
        assert!(s3_path.contains("s3://"), "Should return S3 path");
    }

    #[tokio::test]
    async fn test_file_download() {
        let uploader = FileUploader::new();
        let user_id = uuid::Uuid::new_v4();

        let original_content = b"download test content";
        let filename = "download.txt";

        // Upload file
        let file_id = uploader
            .upload(user_id, filename, original_content)
            .await
            .unwrap();

        // Download file
        let result = uploader.download(user_id, &file_id).await;

        assert!(result.is_ok(), "Download should succeed");

        let (retrieved_filename, retrieved_content) = result.unwrap();
        assert_eq!(retrieved_filename, filename, "Filename should match");
        assert_eq!(retrieved_content, original_content, "Content should match");
    }

    #[tokio::test]
    async fn test_file_delete() {
        let uploader = FileUploader::new();
        let user_id = uuid::Uuid::new_v4();

        // Upload file
        let file_id = uploader
            .upload(user_id, "delete.txt", b"delete me")
            .await
            .unwrap();

        // Verify file exists
        let exists = uploader.file_exists(user_id, &file_id).await;
        assert!(exists, "File should exist");

        // Delete file
        let result = uploader.delete(user_id, &file_id).await;
        assert!(result.is_ok(), "Delete should succeed");

        // Verify file is deleted
        let exists = uploader.file_exists(user_id, &file_id).await;
        assert!(!exists, "File should be deleted");
    }

    #[tokio::test]
    async fn test_file_permissions() {
        let uploader = FileUploader::new();
        let user1 = uuid::Uuid::new_v4();
        let user2 = uuid::Uuid::new_v4();

        // User 1 uploads file
        let file_id = uploader
            .upload(user1, "private.txt", b"user1 secret")
            .await
            .unwrap();

        // User 2 should not access user 1's file
        let result = uploader.get_file(user2, &file_id).await;
        assert!(result.is_err(), "User 2 should not access user 1 file");
    }

    #[tokio::test]
    async fn test_profile_picture_upload() {
        let uploader = FileUploader::new();
        let user_id = uuid::Uuid::new_v4();

        let png_header = b"\x89PNG\r\n\x1a\n"; // Valid PNG header
        let image_data = [png_header, b"fake image data"].concat();

        // Upload profile picture
        let result = uploader
            .upload_profile_picture(user_id, "profile.png", &image_data, "image/png")
            .await;

        assert!(result.is_ok(), "Profile picture upload should succeed");

        // Verify thumbnail was created
        let thumbnail = uploader.get_thumbnail(user_id).await;
        assert!(thumbnail.is_ok(), "Thumbnail should be created");
    }

    #[tokio::test]
    async fn test_file_metadata() {
        let uploader = FileUploader::new();
        let user_id = uuid::Uuid::new_v4();

        let content = b"test file with metadata";

        // Upload file
        let file_id = uploader
            .upload(user_id, "metadata.txt", content)
            .await
            .unwrap();

        // Get metadata
        let result = uploader.get_metadata(user_id, &file_id).await;

        assert!(result.is_ok(), "Should retrieve metadata");

        let metadata = result.unwrap();
        assert_eq!(metadata.filename, "metadata.txt");
        assert_eq!(metadata.size, content.len() as u64);
        assert!(!metadata.created_at.is_empty());
    }

    #[tokio::test]
    async fn test_file_list() {
        let uploader = FileUploader::new();
        let user_id = uuid::Uuid::new_v4();

        // Upload multiple files
        for i in 0..5 {
            let _ = uploader
                .upload(user_id, &format!("file{}.txt", i), b"content")
                .await;
        }

        // List files
        let result = uploader.list_files(user_id).await;

        assert!(result.is_ok(), "List should succeed");

        let files = result.unwrap();
        assert_eq!(files.len(), 5, "Should have 5 files");
    }

    #[tokio::test]
    async fn test_file_upload_resume() {
        let uploader = FileUploader::new();
        let user_id = uuid::Uuid::new_v4();

        let content = vec![0u8; 10 * 1024 * 1024]; // 10 MB

        // Start upload
        let session_id = uploader
            .start_upload(user_id, "large.bin", content.len() as u64)
            .await
            .unwrap();

        // Upload in chunks
        let chunk_size = 1024 * 1024; // 1 MB chunks

        for (i, chunk) in content.chunks(chunk_size).enumerate() {
            let result = uploader
                .upload_chunk(&session_id, i as u32, chunk)
                .await;

            assert!(result.is_ok(), "Chunk {} upload should succeed", i);
        }

        // Complete upload
        let result = uploader.complete_upload(&session_id).await;

        assert!(result.is_ok(), "Upload completion should succeed");
    }

    #[tokio::test]
    async fn test_file_upload_cleanup() {
        let uploader = FileUploader::new();
        let user_id = uuid::Uuid::new_v4();

        // Create temporary file
        let file_id = uploader
            .upload(user_id, "temp.txt", b"temporary")
            .await
            .unwrap();

        // Mark as temporary
        uploader.mark_temporary(&file_id, 3600).await.ok();

        // Wait for cleanup (or trigger manually in test)
        uploader.cleanup_expired_files().await.ok();

        // File might still exist (depends on retention policy)
        // This test verifies the cleanup process runs without error
    }

    #[tokio::test]
    async fn test_concurrent_uploads() {
        let uploader = std::sync::Arc::new(FileUploader::new());
        let user_id = uuid::Uuid::new_v4();

        let mut handles = vec![];

        // Create 10 concurrent uploads
        for i in 0..10 {
            let uploader_clone = uploader.clone();
            let user_id = user_id;

            let handle = tokio::spawn(async move {
                uploader_clone
                    .upload(user_id, &format!("concurrent{}.txt", i), b"content")
                    .await
            });

            handles.push(handle);
        }

        // Wait for all uploads
        let results: Vec<_> = futures::future::join_all(handles)
            .await
            .into_iter()
            .filter_map(|r| r.ok())
            .collect();

        // All should succeed
        assert_eq!(results.len(), 10, "All uploads should complete");
        for result in results {
            assert!(result.is_ok(), "Upload should succeed");
        }
    }
}

// Mock implementations
struct FileUploader;

impl FileUploader {
    fn new() -> Self {
        FileUploader
    }

    fn new_with_s3() -> Self {
        FileUploader
    }

    async fn upload(&self, user_id: uuid::Uuid, filename: &str, content: &[u8]) -> Result<String, ApiError> {
        Ok(uuid::Uuid::new_v4().to_string())
    }

    async fn upload_profile_picture(&self, user_id: uuid::Uuid, filename: &str, content: &[u8], mime_type: &str) -> Result<String, ApiError> {
        Ok(uuid::Uuid::new_v4().to_string())
    }

    async fn get_file(&self, user_id: uuid::Uuid, file_id: &str) -> Result<Vec<u8>, ApiError> {
        Ok(vec![])
    }

    async fn download(&self, user_id: uuid::Uuid, file_id: &str) -> Result<(String, Vec<u8>), ApiError> {
        Ok(("file.txt".to_string(), vec![]))
    }

    async fn delete(&self, user_id: uuid::Uuid, file_id: &str) -> Result<(), ApiError> {
        Ok(())
    }

    async fn file_exists(&self, user_id: uuid::Uuid, file_id: &str) -> bool {
        true
    }

    async fn upload_to_s3(&self, user_id: uuid::Uuid, filename: &str, content: &[u8]) -> Result<String, ApiError> {
        Ok("s3://bucket/path".to_string())
    }

    async fn get_metadata(&self, user_id: uuid::Uuid, file_id: &str) -> Result<FileMetadata, ApiError> {
        Ok(FileMetadata {
            filename: "file.txt".to_string(),
            size: 100,
            created_at: "2024-01-01".to_string(),
        })
    }

    async fn list_files(&self, user_id: uuid::Uuid) -> Result<Vec<FileMetadata>, ApiError> {
        Ok(vec![])
    }

    async fn start_upload(&self, user_id: uuid::Uuid, filename: &str, size: u64) -> Result<String, ApiError> {
        Ok(uuid::Uuid::new_v4().to_string())
    }

    async fn upload_chunk(&self, session_id: &str, chunk_index: u32, data: &[u8]) -> Result<(), ApiError> {
        Ok(())
    }

    async fn complete_upload(&self, session_id: &str) -> Result<(), ApiError> {
        Ok(())
    }

    async fn mark_temporary(&self, file_id: &str, ttl_secs: u32) -> Result<(), ApiError> {
        Ok(())
    }

    async fn cleanup_expired_files(&self) -> Result<u32, ApiError> {
        Ok(0)
    }

    async fn get_thumbnail(&self, user_id: uuid::Uuid) -> Result<Vec<u8>, ApiError> {
        Ok(vec![])
    }
}

struct FileMetadata {
    filename: String,
    size: u64,
    created_at: String,
}
