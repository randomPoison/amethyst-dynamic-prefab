use crate::system::PrefabLoaderSystem;
use amethyst::assets::{Asset, AssetStorage, Handle, PrefabData, ProgressCounter};
use amethyst::core::bundle::SystemBundle;
use amethyst::ecs::*;
use amethyst::shred::*;
use serde::de::DeserializeOwned;
use serde::*;
use std::collections::HashMap;
use std::marker::PhantomData;
use type_uuid::*;
use uuid::Uuid;

mod bundle;
mod loader;
mod system;

pub use crate::bundle::DynamicPrefabBundle;
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

struct ComponentWrapper<T>(PhantomData<T>);

impl<'a, T> SerializeDynamic for ComponentWrapper<T>
where
    T: PrefabData<'a> + Serialize + DeserializeOwned + Send + Sync,
{
    fn instantiate(
        &self,
        data: &ron::Value,
        entity: Entity,
        resources: &DynamicPrefabSystemData,
    ) -> Result<(), String> {
        unimplemented!("Instantiate the component for realsies: {:?}", data);
    }
}

trait SerializeDynamic: Send + Sync {
    fn instantiate(
        &self,
        data: &ron::Value,
        entity: Entity,
        resources: &DynamicPrefabSystemData,
    ) -> Result<(), String>;
}

type DynamicPrefabSystemData<'a> = HashMap<ResourceId, &'a Resource>;

struct DynamicPrefabAccessor {
    reads: Vec<ResourceId>,
    writes: Vec<ResourceId>,
}

impl Accessor for DynamicPrefabAccessor {
    fn try_new() -> Option<Self> {
        None
    }

    fn reads(&self) -> Vec<ResourceId> {
        self.reads.clone()
    }

    fn writes(&self) -> Vec<ResourceId> {
        self.writes.clone()
    }
}
