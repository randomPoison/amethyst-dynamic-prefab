use amethyst::assets::{Asset, AssetStorage, Handle, PrefabData, PrefabError, ProgressCounter};
use amethyst::ecs::*;
use amethyst::shred::ResourceId;
use log::*;
use serde::de::DeserializeOwned;
use serde::*;
use std::collections::HashMap;
use std::marker::PhantomData;
use uuid::Uuid;

mod bundle;
mod loader;
mod system;

pub use crate::bundle::DynamicPrefabBundle;
pub use crate::loader::DynamicPrefabLoader;

type SerializerMap = HashMap<Uuid, Box<dyn SerializeDynamic>>;
type DynamicPrefabStorage = AssetStorage<DynamicPrefab>;

/// The serialized representation of a prefab.
type DynamicPrefabData = Vec<HashMap<Uuid, serde_json::Value>>;

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

impl<T> SerializeDynamic for PhantomData<T>
where
    for<'a> T: PrefabData<'a, Result = ()> + Serialize + DeserializeOwned + Send + Sync,
{
    fn instantiate<'b, 'c, 'd, 'e>(
        &'b self,
        data: &'c serde_json::Value,
        entity: Entity,
        resources: &'d Resources,
        entities: &'e [Entity],
    ) -> Result<(), PrefabError> {
        debug!("Deserializing from {:?}", data);

        let prefab_data = T::deserialize(data.clone())
            .map_err(|err| PrefabError::Custom(amethyst::ecs::error::BoxedErr::new(err)))?;
        let mut system_data = T::SystemData::fetch(resources);
        prefab_data.add_to_entity(entity, &mut system_data, entities)
    }
}

trait SerializeDynamic: Send + Sync {
    fn instantiate(
        &self,
        data: &serde_json::Value,
        entity: Entity,
        resources: &Resources,
        entities: &[Entity],
    ) -> Result<(), PrefabError>;
}

struct DynamicPrefabAccessor {
    reads: Vec<ResourceId>,
    writes: Vec<ResourceId>,
    setup: Vec<Box<dyn SystemDataSetup + Send + Sync>>,
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

trait SystemDataSetup {
    fn setup(&self, res: &mut Resources);
}

impl<T> SystemDataSetup for PhantomData<T>
where
    for<'a> T: PrefabData<'a>,
{
    fn setup(&self, res: &mut Resources) {
        T::SystemData::setup(res);
    }
}
