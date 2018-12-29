use amethyst::assets::*;
use amethyst::core::bundle::SystemBundle;
use amethyst::ecs::shred::Resource;
use amethyst::ecs::*;
use serde::de::DeserializeOwned;
use serde::*;
use std::collections::HashMap;
use std::marker::PhantomData;
use type_uuid::*;
use uuid::Uuid;

mod loader;

pub use crate::loader::PrefabLoader;

#[derive(Default)]
pub struct DynamicPrefabBundle {
    serializer_map: HashMap<Uuid, Box<dyn SerializeDynamic>>,
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

    pub fn register_resource<T>(&mut self)
    where
        T: Resource + Serialize + DeserializeOwned + TypeUuid,
    {
        let uuid = Uuid::from(Uuid::from_u128(T::UUID));
        let serializer = Box::new(ResourceWrapper::<T>(PhantomData)) as Box<SerializeDynamic>;
        self.serializer_map.insert(uuid, serializer);
    }
}

impl<'a, 'b> SystemBundle<'a, 'b> for DynamicPrefabBundle {
    fn build(
        self,
        dispatcher: &mut DispatcherBuilder<'a, 'b>,
    ) -> amethyst::core::bundle::Result<()> {
        Ok(())
    }
}

/// Asset type for dynamic prefabs.
pub struct DynamicPrefab;

impl Asset for DynamicPrefab {
    const NAME: &'static str = "DYNAMIC_PREFAB";
    type Data = Self;
    type HandleStorage = FlaggedStorage<Handle<Self>, DenseVecStorage<Handle<Self>>>;
}

struct ComponentWrapper<T>(PhantomData<T>);

struct ResourceWrapper<T>(PhantomData<T>);

impl<'a, T> SerializeDynamic for ComponentWrapper<T> where
    T: PrefabData<'a> + Serialize + DeserializeOwned + Send + Sync
{
}

impl<T> SerializeDynamic for ResourceWrapper<T> where T: Resource + Serialize + DeserializeOwned {}

trait SerializeDynamic: Send + Sync {}
