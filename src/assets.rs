use bevy::{asset::Asset, log, prelude::*};
use std::{ops::Deref, path::PathBuf};
use thiserror::Error;

pub trait AssetPath {
    fn asset_path<T: AsRef<str>>(&self, path: &T) -> PathBuf;
}

#[derive(Debug, Error)]
pub enum LoadRelativeError {
    #[error("Asset not found: {}", _0)]
    AssetNotFound(String),
}

pub trait LoadRelative {
    fn load_relative<T: Asset, A: AsRef<str>, P: AssetPath>(
        &self,
        path: &A,
        asset_path: &P,
    ) -> Result<Handle<T>, LoadRelativeError>;
}

impl<T: AsRef<str>> AssetPath for T {
    fn asset_path<P: AsRef<str>>(&self, path: &P) -> PathBuf {
        let mut p = PathBuf::from(self.as_ref());
        p.push(path.as_ref());
        p
    }
}

impl<S: Deref<Target = AssetServer>> LoadRelative for S {
    fn load_relative<T: Asset, A: AsRef<str>, P: AssetPath>(
        &self,
        path: &A,
        asset_path: &P,
    ) -> Result<Handle<T>, LoadRelativeError> {
        let path = asset_path.asset_path(path);
        if path.exists() {
            return Ok(self.load(path));
        }

        log::warn!(asset=?path, "Asset not found");
        Err(LoadRelativeError::AssetNotFound(path.display().to_string()))
    }
}
