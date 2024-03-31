use crate::VuekoState;
use bevy::prelude::*;
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt},
};

pub struct VuekoAssetPlugin;
impl Plugin for VuekoAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(VuekoState::Loading)
                .continue_to_state(VuekoState::Playing)
                // .load_collection::<AudioAssets>()
                .load_collection::<ModelAssets>(),
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct ModelAssets {
    #[asset(path = "models/crt.glb#Scene0")]
    pub crt: Handle<Scene>,
}
