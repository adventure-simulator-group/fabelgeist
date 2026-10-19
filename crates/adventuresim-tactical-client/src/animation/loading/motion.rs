use super::{AnimationAssetPath, animation_asset_path};
use bevy::{
    asset::{AssetLoadError, LoadState, io::AssetReaderError},
    gltf::Gltf,
    prelude::*,
    tasks::{IoTaskPool, Task, futures_lite::future},
};

/// Optional motions own their load diagnostics so expected gaps do not emit
/// Bevy's unconditional asset-load errors before pose fallback can run.
pub(crate) enum MotionLoad {
    Pending(Task<Option<Handle<Gltf>>>),
    Ready(Option<Handle<Gltf>>),
}

impl MotionLoad {
    pub(super) fn request(asset_server: &AssetServer, path: &AnimationAssetPath) -> Self {
        let path = animation_asset_path(path);
        let asset_server = asset_server.clone();
        Self::Pending(IoTaskPool::get().spawn(async move {
            match asset_server.load_builder().load_untyped_async(&path).await {
                Ok(handle) => match handle.try_typed::<Gltf>() {
                    Ok(handle) => Some(handle),
                    Err(error) => {
                        error!(path = %path, %error, "Animation motion has the wrong asset type");
                        None
                    }
                },
                Err(AssetLoadError::AssetReaderError(AssetReaderError::NotFound(_))) => {
                    info!(
                        path = %path,
                        "Animation motion is absent; using available pose fallbacks"
                    );
                    None
                }
                Err(error) => {
                    error!(path = %path, %error, "Animation motion failed to load");
                    None
                }
            }
        }))
    }

    /// Retain the source handle even while its current load state is failed.
    pub(in crate::animation) fn source(&self) -> Option<&Handle<Gltf>> {
        match self {
            Self::Ready(handle) => handle.as_ref(),
            Self::Pending(_) => None,
        }
    }

    /// The outer option is pending; the inner option is motion availability.
    pub(super) fn poll(&mut self, asset_server: &AssetServer) -> Option<Option<Handle<Gltf>>> {
        if let Self::Pending(task) = self {
            *self = Self::Ready(future::block_on(future::poll_once(task))?);
        }
        match self {
            Self::Ready(handle) => Some(handle.clone().filter(|handle| {
                !matches!(asset_server.load_state(handle.id()), LoadState::Failed(_))
            })),
            Self::Pending(_) => None,
        }
    }
}
