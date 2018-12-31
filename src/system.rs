use crate::loader::*;
use crate::{DynamicPrefab, DynamicPrefabAccessor, DynamicPrefabStorage, SerializerMap};
use amethyst::{
    assets::*,
    core::{ArcThreadPool, Time},
    ecs::prelude::*,
    shred::*,
};
use log::*;
use shred_derive::*;
use std::ops::Deref;

pub(crate) struct PrefabLoaderSystem {
    serializer_map: SerializerMap,
    accessor: DynamicPrefabAccessor,

    entities: Vec<Entity>,
    finished: Vec<Entity>,
    to_process: BitSet,
    insert_reader: Option<ReaderId<ComponentEvent>>,
    next_tag: u64,
}

impl PrefabLoaderSystem {
    pub(crate) fn new(serializer_map: SerializerMap, accessor: DynamicPrefabAccessor) -> Self {
        PrefabLoaderSystem {
            serializer_map,
            accessor,

            entities: Default::default(),
            finished: Default::default(),
            to_process: Default::default(),
            insert_reader: None,
            next_tag: 0,
        }
    }
}

pub(crate) struct Data<'a> {
    static_data: StaticData<'a>,
    dynamic_data: &'a Resources,
}

#[derive(SystemData)]
struct StaticData<'a> {
    entities: Entities<'a>,
    prefab_storage: Write<'a, DynamicPrefabStorage>,
    prefab_handles: ReadStorage<'a, Handle<DynamicPrefab>>,
    time: Read<'a, Time>,
    pool: ReadExpect<'a, ArcThreadPool>,
    strategy: Option<Read<'a, HotReloadStrategy>>,
    tags: WriteStorage<'a, DynamicPrefabTag>,
}

impl<'a> DynamicSystemData<'a> for Data<'a> {
    type Accessor = DynamicPrefabAccessor;

    fn setup(_accessor: &Self::Accessor, resources: &mut Resources) {
        <StaticData as SystemData>::setup(resources);

        // TODO: Perform setup for any component types if necessary.
    }

    fn fetch(_accessor: &Self::Accessor, resources: &'a Resources) -> Self {
        Data {
            static_data: <StaticData as SystemData>::fetch(resources),
            dynamic_data: resources,
        }
    }
}

impl<'a> System<'a> for PrefabLoaderSystem {
    type SystemData = Data<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let Data {
            static_data: mut data,
            dynamic_data,
        } = data;

        let strategy = data.strategy.as_ref().map(Deref::deref);
        data.prefab_storage.process(
            |mut d| {
                d.tag = Some(self.next_tag);
                self.next_tag += 1;
                if !d.loading() {
                    // TODO: Add support for loading sub-assets in dynamic prefabs.
                    // if !d
                    //     .load_sub_assets(&mut prefab_system_data)
                    //     .chain_err(|| "Failed starting sub asset loading")?
                    {
                        return Ok(ProcessingState::Loaded(d));
                    }
                }
                match d.progress().complete() {
                    Completion::Complete => Ok(ProcessingState::Loaded(d)),
                    Completion::Failed => {
                        error!("Failed loading sub asset: {:?}", d.progress().errors());
                        Err("Failed loading sub asset")?
                    }
                    Completion::Loading => Ok(ProcessingState::Loading(d)),
                }
            },
            data.time.frame_number(),
            &**data.pool,
            strategy,
        );
        data.prefab_handles
            .channel()
            .read(self.insert_reader.as_mut().expect(
                "`PrefabLoaderSystem::setup` was not called before `PrefabLoaderSystem::run`",
            ))
            .for_each(|event| {
                if let ComponentEvent::Inserted(id) = event {
                    self.to_process.add(*id);
                }
            });
        self.finished.clear();
        for (root_entity, handle, _) in
            (&*data.entities, &data.prefab_handles, &self.to_process).join()
        {
            if let Some(prefab) = data.prefab_storage.get(handle) {
                self.finished.push(root_entity);

                // create entities
                self.entities.clear();
                self.entities.push(root_entity);
                for entity_data in prefab.entities.iter().skip(1) {
                    let new_entity = data.entities.create();
                    self.entities.push(new_entity);

                    // TODO: Handle the parent component with all the other components.
                    //
                    // if let Some(parent) = entity_data.parent {
                    //     parents
                    //         .insert(
                    //             new_entity,
                    //             Parent {
                    //                 entity: self.entities[parent],
                    //             },
                    //         )
                    //         .expect("Unable to insert `Parent` for prefab");
                    // }

                    data.tags
                        .insert(
                            new_entity,
                            DynamicPrefabTag::new(prefab.tag.expect(
                                "Unreachable: Every loaded prefab should have a `PrefabTag`",
                            )),
                        )
                        .expect("Unable to insert `PrefabTag` for prefab entity");
                }

                // TODO: Create components.
                for (index, entity_data) in prefab.entities.iter().enumerate() {
                    for (uuid, serialized_component) in entity_data {
                        let serializer = match self.serializer_map.get(uuid) {
                            Some(serializer) => serializer,
                            None => {
                                // TODO: Would be good to also log the name/ID of the prefab so that
                                // users can actually look at the prefab and see the component.
                                error!("No serializer found for UUID {}, did you forget to register a component type?", uuid);
                                continue;
                            }
                        };

                        if let Err(err) = serializer.instantiate(
                            serialized_component,
                            self.entities[index],
                            &dynamic_data,
                            &self.entities,
                        ) {
                            error!("Failed to instantiate component from prefab: {:?}", err);
                        }
                    }
                }
            }
        }

        for entity in &self.finished {
            self.to_process.remove(entity.id());
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(&self.accessor, res);
        self.insert_reader = Some(
            <WriteStorage<Handle<DynamicPrefab>> as SystemData>::fetch(&res).register_reader(),
        );
    }

    fn accessor<'b>(&'b self) -> AccessorCow<'a, 'b, Self> {
        AccessorCow::Ref(&self.accessor)
    }
}
