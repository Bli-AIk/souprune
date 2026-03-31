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
use futures_lite::stream;
use std::collections::HashSet;
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

async fn try_read_first<'a>(
    readers: &'a [FileAssetReader],
    path: &'a Path,
) -> Result<Box<dyn Reader + 'a>, AssetReaderError> {
    for reader in readers {
        if let Ok(result) = reader.read(path).await {
            return Ok(Box::new(result) as Box<dyn Reader + 'a>);
        }
    }
    Err(AssetReaderError::NotFound(path.to_path_buf()))
}

async fn try_read_meta_first<'a>(
    readers: &'a [FileAssetReader],
    path: &'a Path,
) -> Result<Box<dyn Reader + 'a>, AssetReaderError> {
    for reader in readers {
        if let Ok(result) = reader.read_meta(path).await {
            return Ok(Box::new(result) as Box<dyn Reader + 'a>);
        }
    }
    Err(AssetReaderError::NotFound(path.to_path_buf()))
}

/// Merge directory listings from all readers that have the given path.
/// Earlier readers take priority (their entries come first), but all unique
/// entries across all readers are included.
///
/// 合并所有拥有给定路径的读取器的目录列表。
/// 靠前的读取器优先（其条目排在前面），但所有读取器的唯一条目都会被包含。
async fn try_read_directory_merged(
    readers: &[FileAssetReader],
    path: &Path,
) -> Result<Box<PathStream>, AssetReaderError> {
    let mut all_paths = Vec::new();
    let mut seen = HashSet::new();
    let mut found_any = false;

    for reader in readers {
        if let Ok(dir_stream) = reader.read_directory(path).await {
            found_any = true;
            use futures_lite::StreamExt;
            let mut stream = dir_stream;
            while let Some(entry) = stream.next().await {
                if seen.insert(entry.clone()) {
                    all_paths.push(entry);
                }
            }
        }
    }

    if found_any {
        Ok(Box::new(stream::iter(all_paths)))
    } else {
        Err(AssetReaderError::NotFound(path.to_path_buf()))
    }
}

#[allow(refining_impl_trait)]
impl AssetReader for MultiSourceAssetReader {
    fn read<'a>(
        &'a self,
        path: &'a Path,
    ) -> impl ConditionalSendFuture<Output = Result<Box<dyn Reader + 'a>, AssetReaderError>> + 'a
    {
        Box::pin(try_read_first(&self.readers, path))
    }

    fn read_meta<'a>(
        &'a self,
        path: &'a Path,
    ) -> impl ConditionalSendFuture<Output = Result<Box<dyn Reader + 'a>, AssetReaderError>> + 'a
    {
        Box::pin(try_read_meta_first(&self.readers, path))
    }

    fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> impl ConditionalSendFuture<Output = Result<Box<PathStream>, AssetReaderError>> {
        Box::pin(try_read_directory_merged(&self.readers, path))
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
