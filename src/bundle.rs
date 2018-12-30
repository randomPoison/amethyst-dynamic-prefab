use crate::system::PrefabLoaderSystem;
use crate::SerializerMap;
use amethyst::assets::PrefabData;
use amethyst::core::bundle::SystemBundle;
use amethyst::ecs::*;
use serde::de::DeserializeOwned;
use serde::*;
use type_uuid::*;

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
