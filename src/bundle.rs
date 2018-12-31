use crate::system::PrefabLoaderSystem;
use crate::{ComponentWrapper, DynamicPrefabAccessor, SerializeDynamic, SerializerMap};
use amethyst::assets::PrefabData;
use amethyst::core::bundle::SystemBundle;
use amethyst::ecs::*;
use amethyst::shred::*;
use log::*;
use serde::de::DeserializeOwned;
use serde::*;
use type_uuid::*;
use uuid::*;

#[derive(Default)]
pub struct DynamicPrefabBundle {
    serializer_map: SerializerMap,
    reads: Vec<ResourceId>,
    writes: Vec<ResourceId>,
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
        T: 'static + PrefabData<'a> + Serialize + DeserializeOwned + TypeUuid + Send + Sync,
    {
        let uuid = Uuid::from(Uuid::from_bytes(T::UUID));
        debug!("Registering component with UUID {}", uuid);

        let serializer =
            Box::new(ComponentWrapper::<T>(Default::default())) as Box<SerializeDynamic>;
        self.serializer_map.insert(uuid, serializer);

        // Add a record of all the resource types that component needs in order to be
        // instantiated.
        self.reads.extend_from_slice(&T::SystemData::reads());
        self.writes.extend_from_slice(&T::SystemData::writes());
    }
}

impl<'a, 'b> SystemBundle<'a, 'b> for DynamicPrefabBundle {
    fn build(
        self,
        dispatcher: &mut DispatcherBuilder<'a, 'b>,
    ) -> amethyst::core::bundle::Result<()> {
        let accessor = DynamicPrefabAccessor {
            reads: self.reads,
            writes: self.writes,
        };

        dispatcher.add(
            PrefabLoaderSystem::new(self.serializer_map, accessor),
            "",
            &[],
        );
        Ok(())
    }
}
