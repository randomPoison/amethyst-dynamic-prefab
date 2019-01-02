# amethyst-dynamic-prefab

[![Join us on Discord](https://img.shields.io/discord/425678876929163284.svg?logo=discord)](https://discord.gg/GnP5Whs)
[![MIT/Apache](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](COPYING.txt)

An alternate prefab implementation for [Amethyst] that dynamically deserializes
component data based on a UUID value assigned to each component type. This type
of prefab is intended to be authored using the [Amethyst Editor].

## Setup and Usage

Here's an example of how to setup an Amethyst game to load dynamic prefabs:

```rust
use amethyst::{
    assets::JsonFormat,
    core::TransformBundle,
    ecs::*,
    prelude::*,
    renderer::{DrawShaded, PosNormTex},
    utils::application_root_dir,
    Error,
};
use amethyst_dynamic_prefab::*;
use tap::*;

struct AssetsExample;

impl SimpleState for AssetsExample {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        // Use the `DynamicPrefabLoader` resource to begin loading a prefab and
        // get a handle to the data.
        let prefab_handle = data.world.exec(|loader: DynamicPrefabLoader<'_>| {
            loader.load("prefab/example.json", JsonFormat, (), ())
        });

        // Attach the handle to an entity to instantiate the prefab in the world.
        data.world.create_entity().with(prefab_handle).build();
    }
}

fn main() -> Result<(), Error> {
    // Setup the `DynamicPrefabBundle` and register all your component types.
    let prefab_bundle = DynamicPrefabBundle::new()
        .tap(DynamicPrefabBundle::register_default_types);

    let game_data = GameDataBuilder::default()
        .with_bundle(prefab_bundle)?;

    Ok(())
}
```

You can run a basic example by downloading the repository and executing:

```
cargo run --example cube
```

## Custom Components

When creating custom component types, you need to do the following:

* Derive `Serialize` and `Deserialize`.
* Derive `PrefabData`.
* [Generate a UUID](https://www.uuidgenerator.net) and derive the `TypeUuid` trait.
* Register your component with the `DynamicPrefabBundle`.

```rust
use serde::*;
use type_uuid::*;
use amethyst::assets::*;

#[derive(Serialize, Deserialize, PrefabData, TypeUuid)]
#[uuid("c86d0124-92c7-4a72-a6d6-b7bd83184f6e")]
pub struct MyComponent {
    foo: String,
    bar: usize,
}

fn main() -> Result<(), Error> {
    let prefab_bundle = DynamicPrefabBundle::new()
        .tap(DynamicPrefabBundle::register_default_types)
        .tap(DynamicPrefabBundle::register::<MyComponent>);

    let game_data = GameDataBuilder::default()
        .with_bundle(prefab_bundle)?;

    Ok(())
}
```

## Prefab Files

Prefabs are defined in JSON files with the following format:

* The top level of the document is an array.
* Each array element is a dictionary of components.
* The key for each component is the UUID for the component type.
* The value for each component is the serialized data for the component.

```json
[
    {
        "f3d49cc2-c77e-4dc9-9e1f-c01e9279c999": {
            "translation": [
                0.0,
                0.0,
                5.0
            ],
            "scale": [
                2.0,
                2.0,
                2.0
            ]
        }
    },
    {
        "f3d49cc2-c77e-4dc9-9e1f-c01e9279c999": {
            "translation": [
                5.0,
                -20.0,
                15.0
            ]
        },
        "41c40489-269b-4ef5-af7f-675a29473f86": {
            "light": {
                "Point": {
                    "intensity": 100.0,
                    "color": [
                        1.0,
                        1.0,
                        1.0,
                        1.0
                    ],
                    "radius": 1.0
                }
            }
        }
    }
]
```

Users are not expected to write prefab files by hand, rather it will be possible
to use the [Amethyst Editor] to create them.

[Amethyst]: https://www.amethyst.rs/
[Amethyst Editor]: https://github.com/amethyst/amethyst-editor
