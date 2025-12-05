use bevy::asset::io::file::FileAssetReader;
use bevy::asset::io::{AssetReader, AssetReaderError, PathStream, Reader};
use bevy::prelude::*;
use bevy::tasks::ConditionalSendFuture;
use std::path::Path;

/// A composite asset reader that tries to read from multiple sources in order.
///
/// 一个复合资产读取器，按顺序尝试从多个源读取。
pub struct MultiSourceAssetReader {
    readers: Vec<FileAssetReader>,
}

impl MultiSourceAssetReader {
    pub fn new(readers: Vec<FileAssetReader>) -> Self {
        Self { readers }
    }
}

impl AssetReader for MultiSourceAssetReader {
    fn read<'a>(
        &'a self,
        path: &'a Path,
    ) -> impl ConditionalSendFuture<Output = Result<Box<dyn Reader + 'a>, AssetReaderError>> + 'a
    {
        Box::pin(async move {
            for reader in &self.readers {
                match reader.read(path).await {
                    Ok(result) => return Ok(Box::new(result) as Box<dyn Reader + 'a>),
                    Err(e) => {
                        if matches!(e, AssetReaderError::NotFound(_)) {
                            continue;
                        }
                        continue;
                    }
                }
            }
            Err(AssetReaderError::NotFound(path.to_path_buf()))
        })
    }

    fn read_meta<'a>(
        &'a self,
        path: &'a Path,
    ) -> impl ConditionalSendFuture<Output = Result<Box<dyn Reader + 'a>, AssetReaderError>> + 'a
    {
        Box::pin(async move {
            for reader in &self.readers {
                match reader.read_meta(path).await {
                    Ok(result) => return Ok(Box::new(result) as Box<dyn Reader + 'a>),
                    Err(e) => {
                        if matches!(e, AssetReaderError::NotFound(_)) {
                            continue;
                        }
                        continue;
                    }
                }
            }
            Err(AssetReaderError::NotFound(path.to_path_buf()))
        })
    }

    fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> impl ConditionalSendFuture<Output = Result<Box<PathStream>, AssetReaderError>> {
        Box::pin(async move {
            for reader in &self.readers {
                match reader.read_directory(path).await {
                    Ok(result) => return Ok(result),
                    Err(e) => {
                        if matches!(e, AssetReaderError::NotFound(_)) {
                            continue;
                        }
                        continue;
                    }
                }
            }
            Err(AssetReaderError::NotFound(path.to_path_buf()))
        })
    }

    fn is_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> impl ConditionalSendFuture<Output = Result<bool, AssetReaderError>> {
        Box::pin(async move {
            for reader in &self.readers {
                match reader.is_directory(path).await {
                    Ok(true) => return Ok(true),
                    Ok(false) => continue,
                    Err(_) => continue,
                }
            }
            Ok(false)
        })
    }
}
