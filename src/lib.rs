use crate::system::PrefabLoaderSystem;
use amethyst::assets::*;
use amethyst::core::bundle::SystemBundle;
use amethyst::ecs::*;
use serde::de::DeserializeOwned;
use serde::*;
use std::collections::HashMap;
use std::marker::PhantomData;
use type_uuid::*;
use uuid::Uuid;

mod loader;
mod system;

pub use crate::loader::DynamicPrefabLoader;

type SerializerMap = HashMap<Uuid, Box<dyn SerializeDynamic>>;
type DynamicPrefabStorage = AssetStorage<DynamicPrefab>;

/// The serialized representation of a prefab.
type DynamicPrefabData = Vec<HashMap<Uuid, ron::Value>>;

/// Asset type for dynamic prefabs.
pub struct DynamicPrefab {
    tag: Option<u64>,
    entities: DynamicPrefabData,
    counter: Option<ProgressCounter>,
}

impl DynamicPrefab {
    /// Check if sub asset loading have been triggered.
    pub fn loading(&self) -> bool {
        self.counter.is_some()
    }

    /// Get the `ProgressCounter` for the sub asset loading.
    ///
    /// ### Panics
    ///
    /// If sub asset loading has not been triggered.
    pub fn progress(&self) -> &ProgressCounter {
        self.counter
            .as_ref()
            .expect("Sub asset loading has not been triggered")
    }
}

impl Asset for DynamicPrefab {
    const NAME: &'static str = "DYNAMIC_PREFAB";
    type Data = Self;
    type HandleStorage = FlaggedStorage<Handle<Self>, DenseVecStorage<Handle<Self>>>;
}

impl<'a> Deserialize<'a> for DynamicPrefab {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        let data = DynamicPrefabData::deserialize(deserializer)?;
        Ok(DynamicPrefab {
            tag: None,
            entities: data,
            counter: None,
        })
    }
}

#[derive(Default)]
pub struct DynamicPrefabBundle {
    serializer_map: SerializerMap,
}

impl DynamicPrefabBundle {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn register_default_types(&mut self) {
        self.register_component::<amethyst::core::Transform>();
        self.register_component::<amethyst::renderer::CameraPrefab>();
        self.register_component::<amethyst::renderer::LightPrefab>();
    }

    pub fn register_component<'a, T>(&mut self)
    where
        T: PrefabData<'a> + Serialize + DeserializeOwned + TypeUuid,
    {
        // let uuid = Uuid::from(Uuid::from_u128(T::UUID));
        // let serializer = Box::new(ComponentWrapper::<T>(PhantomData)) as Box<SerializeDynamic>;
        // self.serializer_map.insert(uuid, serializer);
    }
}

impl<'a, 'b> SystemBundle<'a, 'b> for DynamicPrefabBundle {
    fn build(
        self,
        dispatcher: &mut DispatcherBuilder<'a, 'b>,
    ) -> amethyst::core::bundle::Result<()> {
        dispatcher.add(PrefabLoaderSystem::new(self.serializer_map), "", &[]);

        Ok(())
    }
}

struct ComponentWrapper<T>(PhantomData<T>);

impl<'a, T> SerializeDynamic for ComponentWrapper<T> where
    T: PrefabData<'a> + Serialize + DeserializeOwned + Send + Sync
{
}

trait SerializeDynamic: Send + Sync {}
