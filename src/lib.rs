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
type IntermediatePrefabData = Vec<HashMap<Uuid, serde_json::Value>>;
type TypedPrefabData = Vec<Vec<Box<dyn DynamicPrefabData>>>;

enum IntermediateDataState {
    Untyped(IntermediatePrefabData),
    Typed(TypedPrefabData),
}

impl IntermediateDataState {
    fn deserialize(self, serializer_map: &SerializerMap) -> Self {
        let mut intermediate_data = match self {
            IntermediateDataState::Untyped(data) => data,
            IntermediateDataState::Typed(data) => return IntermediateDataState::Typed(data),
        };

        let typed_data = intermediate_data
            .drain(..)
            .map(|mut components| {
                components.drain().filter_map(|(uuid, component_data)| {
                    let serializer = match serializer_map.get(&uuid) {
                        Some(serializer) => serializer,
                        None => {
                            // TODO: Would be good to also log the name/ID of the prefab so that
                            // users can actually look at the prefab and see the component.
                            error!("No serializer found for UUID {}, did you forget to register a component type?", uuid);
                            return None;
                        }
                    };

                    match serializer.instantiate(component_data) {
                        Ok(prefab_data) => Some(prefab_data),
                        Err(err) => {
                            error!("Error deserializing component for UUID {}: {:?}", uuid, err);
                            None
                        }
                    }
                }).collect()
            })
            .collect();

        IntermediateDataState::Typed(typed_data)
    }
}

trait DynamicPrefabData: Send + Sync {
    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &Resources,
        entities: &[Entity],
    ) -> Result<(), PrefabError>;

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &Resources,
    ) -> Result<bool, PrefabError>;
}

impl<T> DynamicPrefabData for T
where
    for<'a> T: PrefabData<'a, Result = ()> + Send + Sync,
{
    fn add_to_entity(
        &self,
        entity: Entity,
        resources: &Resources,
        entities: &[Entity],
    ) -> Result<(), PrefabError> {
        let mut system_data = T::SystemData::fetch(resources);
        PrefabData::add_to_entity(self, entity, &mut system_data, entities)
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        resources: &Resources,
    ) -> Result<bool, PrefabError> {
        let mut system_data = T::SystemData::fetch(resources);
        PrefabData::load_sub_assets(self, progress, &mut system_data)
    }
}

/// Asset type for dynamic prefabs.
pub struct DynamicPrefab {
    tag: u64,
    entities: Vec<Vec<Box<dyn DynamicPrefabData>>>,
}

/// Intermediate representation for dynamic prefabs.
///
/// Dynamic prefabs are loaded in two phases: First the raw asset data is
/// deserialized into an intermediate representation that leaves the component
/// data anonymous. Then, the prefab loader system performs a second pass over
/// the data and uses the type UUIDs to fully deserialize the data into their
/// concrete types.
pub struct IntermediateDynamicPrefab {
    tag: Option<u64>,
    entities: IntermediateDataState,
    counter: Option<ProgressCounter>,
}

impl IntermediateDynamicPrefab {
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
    fn load_sub_assets<'a>(&mut self, resources: &Resources) -> Result<bool, PrefabError> {
        let mut ret = false;
        let mut progress = ProgressCounter::default();
        let entities = match &mut self.entities {
            IntermediateDataState::Untyped(_) => {
                unreachable!("Can't load sub-assets until component data has been deserialized")
            }
            IntermediateDataState::Typed(entities) => entities,
        };
        for entity in entities {
            for dyn_prefab_data in entity {
                if dyn_prefab_data.load_sub_assets(&mut progress, resources)? {
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
    type Data = IntermediateDynamicPrefab;
    type HandleStorage = FlaggedStorage<Handle<Self>, DenseVecStorage<Handle<Self>>>;
}

impl From<IntermediateDynamicPrefab> for DynamicPrefab {
    fn from(from: IntermediateDynamicPrefab) -> Self {
        let tag = from
            .tag
            .expect("Tag wasn't initialized on intermediate data");
        let entities = match from.entities {
            IntermediateDataState::Typed(entities) => entities,
            _ => panic!("Intermediate prefab data wasn't deserialized"),
        };

        Self { tag, entities }
    }
}

impl<'a> Deserialize<'a> for IntermediateDynamicPrefab {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        let data = IntermediatePrefabData::deserialize(deserializer)?;
        Ok(Self {
            tag: None,
            entities: IntermediateDataState::Untyped(data),
            counter: None,
        })
    }
}

impl<T> SerializeDynamic for PhantomData<T>
where
    for<'a> T: 'static + PrefabData<'a, Result = ()> + Serialize + DeserializeOwned + Send + Sync,
{
    fn instantiate(
        &self,
        data: serde_json::Value,
    ) -> Result<Box<dyn DynamicPrefabData>, PrefabError> {
        debug!("Deserializing from {:?}", data);

        let prefab_data = T::deserialize(data.clone())
            .map_err(|err| PrefabError::Custom(amethyst::ecs::error::BoxedErr::new(err)))?;
        Ok(Box::new(prefab_data) as Box<dyn DynamicPrefabData>)
    }
}

trait SerializeDynamic: Send + Sync {
    fn instantiate(
        &self,
        data: serde_json::Value,
    ) -> Result<Box<dyn DynamicPrefabData>, PrefabError>;
}

struct DynamicPrefabAccessor {
    reads: Vec<ResourceId>,
    writes: Vec<ResourceId>,
    setup: Vec<Box<dyn SystemDataSetup>>,
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

trait SystemDataSetup: Send + Sync {
    fn setup(&self, res: &mut Resources);
}

impl<T> SystemDataSetup for PhantomData<T>
where
    for<'a> T: PrefabData<'a> + Send + Sync,
{
    fn setup(&self, res: &mut Resources) {
        T::SystemData::setup(res);
    }
}
