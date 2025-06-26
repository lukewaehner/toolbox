use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tar::Builder;
use flate2::Compression;
use flate2::write::GzEncoder;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub created_at: i64,
    pub version: String,
    pub backup_type: BackupType,
    pub file_count: usize,
    pub total_size_bytes: u64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupType {
    Full,
    ConfigOnly,
    TasksOnly,
    PasswordsOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub filename: String,
    pub metadata: BackupMetadata,
    pub file_path: PathBuf,
}

pub struct BackupManager {
    backup_dir: PathBuf,
    data_dir: PathBuf,
}

impl BackupManager {
    pub fn new(backup_dir: &str, data_dir: &str) -> io::Result<Self> {
        let backup_path = PathBuf::from(backup_dir);
        let data_path = PathBuf::from(data_dir);
        
        // Create backup directory if it doesn't exist
        fs::create_dir_all(&backup_path)?;
        
        Ok(Self {
            backup_dir: backup_path,
            data_dir: data_path,
        })
    }

    pub fn create_backup(&self, backup_type: BackupType, description: String) -> io::Result<BackupInfo> {
        let timestamp = Utc::now();
        let backup_filename = format!(
            "toolbox_backup_{}_{}.tar.gz", 
            timestamp.format("%Y%m%d_%H%M%S"),
            match backup_type {
                BackupType::Full => "full",
                BackupType::ConfigOnly => "config",
                BackupType::TasksOnly => "tasks",
                BackupType::PasswordsOnly => "passwords",
            }
        );
        
        let backup_path = self.backup_dir.join(&backup_filename);
        let tar_gz = File::create(&backup_path)?;
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = Builder::new(enc);

        let mut file_count = 0;
        let mut total_size = 0;

        // Add files based on backup type
        match backup_type {
            BackupType::Full => {
                self.add_all_files_to_tar(&mut tar, &mut file_count, &mut total_size)?;
            },
            BackupType::ConfigOnly => {
                self.add_config_files_to_tar(&mut tar, &mut file_count, &mut total_size)?;
            },
            BackupType::TasksOnly => {
                self.add_task_files_to_tar(&mut tar, &mut file_count, &mut total_size)?;
            },
            BackupType::PasswordsOnly => {
                self.add_password_files_to_tar(&mut tar, &mut file_count, &mut total_size)?;
            },
        }

        // Finalize the tar archive
        tar.into_inner()?.finish()?;

        // Create metadata
        let metadata = BackupMetadata {
            created_at: timestamp.timestamp(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            backup_type: backup_type.clone(),
            file_count,
            total_size_bytes: total_size,
            description,
        };

        // Save metadata file
        let metadata_filename = format!("{}.meta", backup_filename);
        let metadata_path = self.backup_dir.join(&metadata_filename);
        let metadata_content = serde_json::to_string_pretty(&metadata)?;
        fs::write(&metadata_path, metadata_content)?;

        Ok(BackupInfo {
            filename: backup_filename,
            metadata,
            file_path: backup_path,
        })
    }

    pub fn list_backups(&self) -> io::Result<Vec<BackupInfo>> {
        let mut backups = Vec::new();
        
        for entry in fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(extension) = path.extension() {
                if extension == "meta" {
                    if let Ok(metadata_content) = fs::read_to_string(&path) {
                        if let Ok(metadata) = serde_json::from_str::<BackupMetadata>(&metadata_content) {
                            let backup_filename = path.file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or_default()
                                .to_string();
                            
                            let backup_path = self.backup_dir.join(&backup_filename);
                            
                            if backup_path.exists() {
                                backups.push(BackupInfo {
                                    filename: backup_filename,
                                    metadata,
                                    file_path: backup_path,
                                });
                            }
                        }
                    }
                }
            }
        }
        
        // Sort by creation time (newest first)
        backups.sort_by(|a, b| b.metadata.created_at.cmp(&a.metadata.created_at));
        
        Ok(backups)
    }

    pub fn restore_backup(&self, backup_info: &BackupInfo, target_dir: Option<&str>) -> io::Result<()> {
        let restore_dir = target_dir.map(PathBuf::from)
            .unwrap_or_else(|| self.data_dir.clone());
        
        // Create restore directory if it doesn't exist
        fs::create_dir_all(&restore_dir)?;
        
        let tar_gz = File::open(&backup_info.file_path)?;
        let tar = flate2::read::GzDecoder::new(tar_gz);
        let mut archive = tar::Archive::new(tar);
        
        archive.unpack(&restore_dir)?;
        
        Ok(())
    }

    pub fn delete_backup(&self, backup_info: &BackupInfo) -> io::Result<()> {
        // Delete the backup file
        if backup_info.file_path.exists() {
            fs::remove_file(&backup_info.file_path)?;
        }
        
        // Delete the metadata file
        let metadata_path = self.backup_dir.join(format!("{}.meta", backup_info.filename));
        if metadata_path.exists() {
            fs::remove_file(&metadata_path)?;
        }
        
        Ok(())
    }

    pub fn cleanup_old_backups(&self, keep_count: usize) -> io::Result<Vec<String>> {
        let backups = self.list_backups()?;
        let mut deleted = Vec::new();
        
        if backups.len() > keep_count {
            let to_delete = &backups[keep_count..];
            
            for backup in to_delete {
                self.delete_backup(backup)?;
                deleted.push(backup.filename.clone());
            }
        }
        
        Ok(deleted)
    }

    pub fn get_backup_stats(&self) -> io::Result<(usize, u64)> {
        let backups = self.list_backups()?;
        let total_count = backups.len();
        let total_size = backups.iter()
            .map(|b| b.metadata.total_size_bytes)
            .sum();
        
        Ok((total_count, total_size))
    }

    fn add_all_files_to_tar(&self, tar: &mut Builder<GzEncoder<File>>, file_count: &mut usize, total_size: &mut u64) -> io::Result<()> {
        self.add_config_files_to_tar(tar, file_count, total_size)?;
        self.add_task_files_to_tar(tar, file_count, total_size)?;
        self.add_password_files_to_tar(tar, file_count, total_size)?;
        Ok(())
    }

    fn add_config_files_to_tar(&self, tar: &mut Builder<GzEncoder<File>>, file_count: &mut usize, total_size: &mut u64) -> io::Result<()> {
        let config_files = [
            "config/app.json",
            "email_config.json",
            "sms_config.json",
            "outlook_config.json",
        ];

        for file in &config_files {
            let file_path = self.data_dir.join(file);
            if file_path.exists() {
                let metadata = fs::metadata(&file_path)?;
                tar.append_path_with_name(&file_path, file)?;
                *file_count += 1;
                *total_size += metadata.len();
            }
        }

        Ok(())
    }

    fn add_task_files_to_tar(&self, tar: &mut Builder<GzEncoder<File>>, file_count: &mut usize, total_size: &mut u64) -> io::Result<()> {
        let task_files = [
            "tasks.json",
        ];

        for file in &task_files {
            let file_path = self.data_dir.join(file);
            if file_path.exists() {
                let metadata = fs::metadata(&file_path)?;
                tar.append_path_with_name(&file_path, file)?;
                *file_count += 1;
                *total_size += metadata.len();
            }
        }

        Ok(())
    }

    fn add_password_files_to_tar(&self, tar: &mut Builder<GzEncoder<File>>, file_count: &mut usize, total_size: &mut u64) -> io::Result<()> {
        let password_files = [
            "passwords.json",
        ];

        for file in &password_files {
            let file_path = self.data_dir.join(file);
            if file_path.exists() {
                let metadata = fs::metadata(&file_path)?;
                tar.append_path_with_name(&file_path, file)?;
                *file_count += 1;
                *total_size += metadata.len();
            }
        }

        Ok(())
    }

    pub fn auto_backup(&self, backup_type: BackupType) -> io::Result<Option<BackupInfo>> {
        // Check if we need to create a backup based on time since last backup
        let backups = self.list_backups()?;
        let now = Utc::now().timestamp();
        
        // Auto backup if no backup exists or last backup is older than 24 hours
        let should_backup = backups.is_empty() || 
            backups.iter()
                .filter(|b| matches!(b.metadata.backup_type, BackupType::Full))
                .next()
                .map(|b| now - b.metadata.created_at > 86400) // 24 hours
                .unwrap_or(true);

        if should_backup {
            let description = format!("Automatic backup created at {}", 
                DateTime::from_timestamp(now, 0)
                    .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap())
                    .format("%Y-%m-%d %H:%M:%S UTC"));
            
            Ok(Some(self.create_backup(backup_type, description)?))
        } else {
            Ok(None)
        }
    }

    pub fn verify_backup(&self, backup_info: &BackupInfo) -> io::Result<bool> {
        // Check if backup file exists and is readable
        if !backup_info.file_path.exists() {
            return Ok(false);
        }

        // Try to open and read the archive header
        let tar_gz = File::open(&backup_info.file_path)?;
        let tar = flate2::read::GzDecoder::new(tar_gz);
        let mut archive = tar::Archive::new(tar);
        
        // Count entries in archive
        let entry_count = archive.entries()?.count();
        
        // Verify entry count matches metadata
        Ok(entry_count == backup_info.metadata.file_count)
    }
}

pub fn format_backup_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    format!("{:.1} {}", size, UNITS[unit_index])
}

pub fn format_backup_date(timestamp: i64) -> String {
    DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap())
        .format("%Y-%m-%d %H:%M:%S UTC")
        .to_string()
} 