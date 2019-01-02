use crate::system::PrefabLoaderSystem;
use crate::{DynamicPrefabAccessor, SerializeDynamic, SerializerMap, SystemDataSetup};
use amethyst::assets::PrefabData;
use amethyst::core::bundle::SystemBundle;
use amethyst::ecs::*;
use amethyst::shred::*;
use log::*;
use serde::de::DeserializeOwned;
use serde::*;
use std::marker::PhantomData;
use type_uuid::amethyst_types::*;
use type_uuid::*;
use uuid::*;

#[derive(Default)]
pub struct DynamicPrefabBundle {
    serializer_map: SerializerMap,
    reads: Vec<ResourceId>,
    writes: Vec<ResourceId>,
    setup: Vec<Box<dyn SystemDataSetup>>,
}

impl DynamicPrefabBundle {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn register_default_types(&mut self) {
        self.register_component::<amethyst::core::Transform>();
        self.register_component::<amethyst::renderer::CameraPrefab>();
        self.register_component::<amethyst::renderer::LightPrefab>();
        self.register_component::<ObjGraphics>();
    }

    pub fn register_component<T>(&mut self)
    where
        T: 'static,
        for<'a> T:
            PrefabData<'a, Result = ()> + Serialize + DeserializeOwned + TypeUuid + Send + Sync,
    {
        let uuid = Uuid::from(Uuid::from_bytes(T::UUID));
        debug!("Registering component with UUID {}", uuid);

        let serializer = Box::new(PhantomData::<T>) as Box<dyn SerializeDynamic>;
        self.serializer_map.insert(uuid, serializer);

        // Add a record of all the resource types that component needs in order to be
        // instantiated.
        self.reads.extend_from_slice(&T::SystemData::reads());
        self.writes.extend_from_slice(&T::SystemData::writes());

        let setup = Box::new(PhantomData::<T>) as Box<dyn SystemDataSetup>;
        self.setup.push(setup);
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
            setup: self.setup,
        };

        dispatcher.add(
            PrefabLoaderSystem::new(self.serializer_map, accessor),
            "",
            &[],
        );
        Ok(())
    }
}
