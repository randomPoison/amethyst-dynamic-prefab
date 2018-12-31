//! Demonstrates loading prefabs using the Amethyst engine.

use amethyst::{
    assets::RonFormat,
    core::TransformBundle,
    ecs::*,
    prelude::*,
    renderer::{DrawShaded, PosNormTex},
    utils::application_root_dir,
    Error,
};
use amethyst_dynamic_prefab::*;
use amethyst_editor_sync::*;
use tap::*;

// type MyPrefabData = BasicScenePrefab<Vec<PosNormTex>>;

struct AssetsExample;

impl SimpleState for AssetsExample {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let prefab_handle = data.world.exec(|loader: DynamicPrefabLoader<'_>| {
            loader.load("prefab/example.ron", RonFormat, (), ())
        });
        data.world.create_entity().with(prefab_handle).build();
    }
}

/// Wrapper around the main, so we can return errors easily.
fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir();

    // Add our meshes directory to the asset loader.
    let resources_directory = format!("{}/examples/assets", app_root);

    let display_config_path = format!("{}/examples/prefab/resources/display_config.ron", app_root);

    let prefab_bundle = DynamicPrefabBundle::new().tap(DynamicPrefabBundle::register_default_types);
    let editor_bundle = SyncEditorBundle::new().tap(SyncEditorBundle::sync_default_types);

    let game_data = GameDataBuilder::default()
        .with_bundle(prefab_bundle)?
        .with_bundle(editor_bundle)?
        .with_bundle(TransformBundle::new())?
        .with_basic_renderer(display_config_path, DrawShaded::<PosNormTex>::new(), false)?;

    let mut game = Application::new(resources_directory, AssetsExample, game_data)?;
    game.run();
    Ok(())
}
