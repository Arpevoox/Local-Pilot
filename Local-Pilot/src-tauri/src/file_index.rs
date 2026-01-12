//! 本地文件索引模块
//! 使用DuckDB创建和维护本地文件索引

use duckdb::{Connection, params, types::Value};
use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use directories::UserDirs;
use tauri::AppHandle;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::OnceCell;

/// 文件信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub extension: Option<String>,
    pub size: u64,
    pub modified: String, // ISO 8601格式
    pub created: String,  // ISO 8601格式
    pub is_directory: bool,
}

/// 文件索引器结构
pub struct FileIndexer {
    db_connection: Arc<Mutex<Connection>>,
}

impl FileIndexer {
    /// 创建新的文件索引器
    pub fn new(db_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open(db_path)?;
        
        // 创建文件表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS files (
                path TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                extension TEXT,
                size INTEGER,
                modified TEXT,
                created TEXT,
                is_directory BOOLEAN
            )",
            [],
        )?;
        
        Ok(Self {
            db_connection: Arc::new(Mutex::new(conn)),
        })
    }

    /// 扫描指定目录并将文件信息添加到索引
    pub fn scan_directory(&self, dir_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.db_connection.lock().unwrap();
        
        for entry in WalkDir::new(dir_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() || entry.file_type().is_dir() {
                if let Some(file_info) = self.get_file_info(&entry.path())? {
                    // 插入或更新文件信息
                    conn.execute(
                        "INSERT OR REPLACE INTO files (path, name, extension, size, modified, created, is_directory) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        params![
                            file_info.path,
                            file_info.name,
                            file_info.extension,
                            file_info.size as i64,
                            file_info.modified,
                            file_info.created,
                            file_info.is_directory
                        ],
                    )?;
                }
            }
        }
        
        Ok(())
    }

    /// 从路径获取文件信息
    fn get_file_info(&self, path: &Path) -> Result<Option<FileInfo>, Box<dyn std::error::Error>> {
        match fs::metadata(path) {
            Ok(metadata) => {
                let file_type = metadata.file_type();
                let is_directory = file_type.is_dir();
                
                // 获取文件名和扩展名
                let name = path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                
                let extension = path.extension()
                    .map(|ext| ext.to_string_lossy().to_string());
                
                // 获取时间信息
                let modified = metadata.modified()
                    .ok()
                    .map(|t| format!("{:?}", t))
                    .unwrap_or_default();
                
                let created = metadata.created()
                    .ok()
                    .map(|t| format!("{:?}", t))
                    .unwrap_or_default();
                
                let size = if is_directory {
                    0 // 目录大小为0
                } else {
                    metadata.len()
                };
                
                Ok(Some(FileInfo {
                    path: path.to_string_lossy().to_string(),
                    name,
                    extension,
                    size,
                    modified,
                    created,
                    is_directory,
                }))
            }
            Err(_) => Ok(None), // 无法访问的文件，跳过
        }
    }

    /// 搜索文件
    pub fn search_files(&self, query: &str) -> Result<Vec<FileInfo>, Box<dyn std::error::Error>> {
        let conn = self.db_connection.lock().unwrap();
        
        let mut stmt = conn.prepare(query)?;
        let file_iter = stmt.query_map([], |row| {
            Ok(FileInfo {
                path: row.get(0)?,
                name: row.get(1)?,
                extension: row.get(2)?,
                size: row.get(3)?,
                modified: row.get(4)?,
                created: row.get(5)?,
                is_directory: row.get(6)?,
            })
        })?;
        
        let mut files = Vec::new();
        for file_result in file_iter {
            files.push(file_result?);
        }
        
        Ok(files)
    }

    /// 搜索文件名
    pub fn search_by_filename(&self, filename_pattern: &str) -> Result<Vec<FileInfo>, Box<dyn std::error::Error>> {
        let conn = self.db_connection.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT path, name, extension, size, modified, created, is_directory 
             FROM files 
             WHERE name LIKE ?1"
        )?;
        
        let file_iter = stmt.query_map([format!("%{}%", filename_pattern)], |row| {
            Ok(FileInfo {
                path: row.get(0)?,
                name: row.get(1)?,
                extension: row.get(2)?,
                size: row.get(3)?,
                modified: row.get(4)?,
                created: row.get(5)?,
                is_directory: row.get(6)?,
            })
        })?;
        
        let mut files = Vec::new();
        for file_result in file_iter {
            files.push(file_result?);
        }
        
        Ok(files)
    }

    /// 搜索文件扩展名
    pub fn search_by_extension(&self, extension: &str) -> Result<Vec<FileInfo>, Box<dyn std::error::Error>> {
        let conn = self.db_connection.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT path, name, extension, size, modified, created, is_directory 
             FROM files 
             WHERE extension = ?1"
        )?;
        
        let file_iter = stmt.query_map([extension], |row| {
            Ok(FileInfo {
                path: row.get(0)?,
                name: row.get(1)?,
                extension: row.get(2)?,
                size: row.get(3)?,
                modified: row.get(4)?,
                created: row.get(5)?,
                is_directory: row.get(6)?,
            })
        })?;
        
        let mut files = Vec::new();
        for file_result in file_iter {
            files.push(file_result?);
        }
        
        Ok(files)
    }

    /// 获取数据库连接
    pub fn get_connection(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.db_connection)
    }
}

/// 初始化文件索引器
pub fn initialize_file_indexer(app_handle: &AppHandle) -> Result<FileIndexer, Box<dyn std::error::Error>> {
    // 获取应用数据目录
    let app_data_dir = app_handle.path().app_data_dir()?;
    let db_path = app_data_dir.join("file_index.db");
    
    // 创建文件索引器
    let indexer = FileIndexer::new(db_path.to_str().unwrap())?;
    
    // 获取用户目录
    if let Some(user_dirs) = UserDirs::new() {
        // 扫描常见目录
        if let Some(download_dir) = user_dirs.download_dir() {
            if download_dir.exists() {
                indexer.scan_directory(download_dir.to_str().unwrap())?;
            }
        }
        
        if let Some(desktop_dir) = user_dirs.desktop_dir() {
            if desktop_dir.exists() {
                indexer.scan_directory(desktop_dir.to_str().unwrap())?;
            }
        }
        
        if let Some(documents_dir) = user_dirs.document_dir() {
            if documents_dir.exists() {
                indexer.scan_directory(documents_dir.to_str().unwrap())?;
            }
        }
    }
    
    Ok(indexer)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_file_indexer_creation() {
        let temp_dir = std::env::temp_dir().join("test_file_indexer.db");
        let indexer = FileIndexer::new(temp_dir.to_str().unwrap());
        assert!(indexer.is_ok());
    }
}