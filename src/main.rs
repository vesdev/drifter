use anyhow::Ok;
use assets::VuekoAssetPlugin;
use bevy::prelude::*;
use vueko::VuekoPlugin;

mod assets;
mod event;
mod fx;
mod vueko;

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum VuekoState {
    #[default]
    Loading,
    Playing,
}

fn main() -> anyhow::Result<()> {
    let receiver = event::create_receiver()?;

    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<VuekoState>()
        .insert_non_send_resource(receiver)
        .add_plugins((VuekoAssetPlugin, VuekoPlugin))
        .run();
    Ok(())
}
