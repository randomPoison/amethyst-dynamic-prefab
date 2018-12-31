use crate::{DynamicPrefab, DynamicPrefabStorage};
use amethyst::{assets::*, ecs::prelude::*};
use shred_derive::*;

#[derive(SystemData)]
pub struct DynamicPrefabLoader<'a> {
    loader: ReadExpect<'a, Loader>,
    storage: Read<'a, DynamicPrefabStorage>,
}

impl<'a> DynamicPrefabLoader<'a> {
    pub fn load<F, N, P>(
        &self,
        name: N,
        format: F,
        options: F::Options,
        progress: P,
    ) -> Handle<DynamicPrefab>
    where
        F: Format<DynamicPrefab>,
        N: Into<String>,
        P: Progress,
    {
        self.loader
            .load(name, format, options, progress, &self.storage)
    }
}

/// Tag placed on entities created by the prefab system.
///
/// The tag value match the tag value of the `Prefab` the `Entity` was created from.
pub struct DynamicPrefabTag {
    tag: u64,
}

impl DynamicPrefabTag {
    /// Create a new tag
    pub fn new(tag: u64) -> Self {
        DynamicPrefabTag { tag }
    }

    /// Get the tag
    pub fn tag(&self) -> u64 {
        self.tag
    }
}

impl Component for DynamicPrefabTag {
    type Storage = DenseVecStorage<Self>;
}
