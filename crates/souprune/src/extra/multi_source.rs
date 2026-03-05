//! # multi_source.rs
//!
//! # multi_source.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module provides a custom asset reader that supports multiple source directories.
//!
//! 该模块提供了一个支持多个源目录的自定义资产读取器。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It implements `MultiSourceAssetReader` to allow cascading lookups for assets.
//!
//! 它实现了 `MultiSourceAssetReader` 以允许对资产进行级联查找。

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

#[allow(refining_impl_trait)]
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
