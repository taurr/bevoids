mod atlas;
mod audio;
mod bounds;
mod font;
mod texture;

pub use atlas::*;
pub use audio::*;
pub use bounds::*;
pub use font::*;
pub use texture::*;

pub use embedded_assets::*;

mod embedded_assets {
    // TODO: expand to handle multiple protocols - use API to 'add' protocols as extensions
    // TODO: let use of rust-embed be behind feature
    // TODO: extract to own crate
    // TODO: add testcases
    // TODO: add documentation
    // TODO: release on crates.io
    use std::{
        marker::PhantomData,
        path::{Path, PathBuf},
    };

    use bevy::{
        asset::{create_platform_default_asset_io, AssetIo, AssetIoError, BoxedFuture},
        prelude::{AppBuilder, AssetServer, Plugin},
        tasks::IoTaskPool,
    };
    use itertools::Itertools;
    use rust_embed::RustEmbed;

    pub struct EmbeddedAssetPlugin<T>(PhantomData<T>);

    impl<T> EmbeddedAssetPlugin<T>
    where
        T: 'static + RustEmbed + Send + Sync,
    {
        pub fn new() -> Self {
            Self(PhantomData::default())
        }
    }

    impl<T> Default for EmbeddedAssetPlugin<T>
    where
        T: 'static + RustEmbed + Send + Sync,
    {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<T> Plugin for EmbeddedAssetPlugin<T>
    where
        T: 'static + RustEmbed + Send + Sync,
    {
        fn build(&self, app: &mut AppBuilder) {
            let task_pool = app
                .world()
                .get_resource::<IoTaskPool>()
                .expect("`IoTaskPool` resource not found.")
                .0
                .clone();

            let asset_io = Box::new(EmbeddedAssetIo::<T> {
                default_io: create_platform_default_asset_io(app),
                _embed_resources: PhantomData::default(),
            });
            let asset_server = AssetServer::with_boxed_io(asset_io, task_pool);

            app.insert_resource(asset_server);
        }
    }

    pub struct EmbeddedAssetIo<T> {
        default_io: Box<dyn AssetIo>,
        _embed_resources: PhantomData<T>,
    }

    impl<T> AssetIo for EmbeddedAssetIo<T>
    where
        T: 'static + RustEmbed + Send + Sync,
    {
        fn load_path<'a>(
            &'a self,
            path: &'a Path,
        ) -> BoxedFuture<'a, Result<Vec<u8>, AssetIoError>> {
            if let Some(file) = T::get(path.to_str().expect("Expected a path")) {
                Box::pin(async move { Ok(Vec::<u8>::from(file.data)) })
            } else {
                self.default_io.load_path(path)
            }
        }

        fn read_directory(
            &self,
            path: &Path,
        ) -> Result<Box<dyn Iterator<Item = PathBuf>>, AssetIoError> {
            let embedded = T::iter()
                .filter({
                    let str_path = path
                        .to_str()
                        .expect("path is not valid unicode")
                        .to_string();
                    move |x| x.starts_with(&str_path)
                })
                .map(|x| PathBuf::from(x.to_string()));
            let files = self.default_io.read_directory(path)?;
            let chained = embedded.chain(files).unique();
            Ok(Box::new(chained))
        }

        fn watch_path_for_changes(&self, path: &Path) -> Result<(), AssetIoError> {
            self.default_io.watch_path_for_changes(path)
        }

        fn watch_for_changes(&self) -> Result<(), AssetIoError> {
            self.default_io.watch_for_changes()
        }

        fn is_directory(&self, path: &Path) -> bool {
            self.default_io.is_directory(path)
        }
    }
}
