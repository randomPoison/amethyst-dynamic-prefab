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

    /// Trigger sub asset loading for the asset
    fn load_sub_assets<'a>(
        &mut self,
        serializer_map: &SerializerMap,
        resources: &Resources,
    ) -> Result<bool, PrefabError> {
        let mut ret = false;
        let mut progress = ProgressCounter::default();
        for entity in &mut self.entities {
            for (uuid, component_data) in entity {
                let serializer = match serializer_map.get(uuid) {
                    Some(serializer) => serializer,
                    None => {
                        // TODO: Would be good to also log the name/ID of the prefab so that
                        // users can actually look at the prefab and see the component.
                        error!("No serializer found for UUID {}, did you forget to register a component type?", uuid);
                        continue;
                    }
                };

                if serializer.load_sub_assets(component_data, resources, &mut progress)? {
                    ret = true;
                }
            }
        }
        self.counter = Some(progress);
        Ok(ret)
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
    fn instantiate(
        &self,
        data: &serde_json::Value,
        entity: Entity,
        resources: &Resources,
        entities: &[Entity],
    ) -> Result<(), PrefabError> {
        debug!("Deserializing from {:?}", data);

        let prefab_data = T::deserialize(data.clone())
            .map_err(|err| PrefabError::Custom(amethyst::ecs::error::BoxedErr::new(err)))?;
        let mut system_data = T::SystemData::fetch(resources);
        prefab_data.add_to_entity(entity, &mut system_data, entities)
    }

    fn load_sub_assets(
        &self,
        data: &serde_json::Value,
        resources: &Resources,
        progress: &mut ProgressCounter,
    ) -> Result<bool, PrefabError> {
        debug!("Loading sub-assets for {:?}", data);

        let mut prefab_data = T::deserialize(data.clone())
            .map_err(|err| PrefabError::Custom(amethyst::ecs::error::BoxedErr::new(err)))?;
        let mut system_data = T::SystemData::fetch(resources);
        prefab_data.load_sub_assets(progress, &mut system_data)
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

    fn load_sub_assets(
        &self,
        data: &serde_json::Value,
        resources: &Resources,
        progress: &mut ProgressCounter,
    ) -> Result<bool, PrefabError>;
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
