use std::path::PathBuf;

use bevy::{asset::Asset, log, prelude::*};

pub trait RelativeAssetLoader {
    fn load_relative<T: Asset>(&self, relative_to: &Option<String>, asset: &str) -> Handle<T>;
    fn attempt_relative<T: Asset>(
        &self,
        relative_to: &Option<String>,
        asset: &str,
    ) -> Option<Handle<T>>;
}

fn find_asset(base_path: &Option<String>, asset: &str) -> Result<PathBuf, PathBuf> {
    let mut base_path = base_path.as_ref().map_or_else(
        || PathBuf::from(std::env::current_dir().unwrap()),
        PathBuf::from,
    );

    base_path.push(asset);
    if !base_path.exists() {
        let mut buf = base_path.parent().unwrap().to_owned();
        buf.push("assets");
        buf.push(asset);
        if buf.exists() {
            Ok(buf)
        } else {
            Err(base_path)
        }
    } else {
        Ok(base_path)
    }
}

impl<'a> RelativeAssetLoader for AssetServer {
    fn load_relative<T: Asset>(&self, relative_to: &Option<String>, asset: &str) -> Handle<T> {
        match find_asset(relative_to, asset) {
            Ok(p) => {
                let handle = self.load(p.clone());
                log::trace!("loading texture: {}", p.display());
                log::trace!("texture id: {:?}", handle.id);
                handle
            }
            Err(p) => {
                log::warn!("Texture not found: {}", p.display());
                self.load(p)
            }
        }
    }

    fn attempt_relative<T: Asset>(
        &self,
        relative_to: &Option<String>,
        asset: &str,
    ) -> Option<Handle<T>> {
        match find_asset(relative_to, asset) {
            Ok(p) => {
                let handle = self.load(p.clone());
                log::trace!("loading texture: {}", p.display());
                log::trace!("texture id: {:?}", handle.id);
                Some(handle)
            }
            Err(p) => {
                log::trace!("Texture not found: {}", p.display());
                None
            }
        }
    }
}
