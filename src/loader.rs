use crate::{DynamicPrefab, SerializerMap};
use amethyst::assets::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use uuid::*;

pub struct PrefabLoader {
    pub(crate) serializer_map: SerializerMap,
}

impl PrefabLoader {
    pub(crate) fn new() -> Self {
        PrefabLoader {
            serializer_map: Default::default(),
        }
    }

    pub fn load<P>(&self, path: P) -> Handle<DynamicPrefab>
    where
        P: AsRef<Path>,
    {
        let file = File::open(path).expect("Failed to read prefab file");
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader
            .read_to_string(&mut contents)
            .expect("Failed to read prefab to string");

        let partial_data: DynamicPrefabData =
            ron::de::from_str(&contents).expect("Failed to deserialize dynamic prefab data");

        println!("Prefab data: {:#?}", partial_data);

        unimplemented!("Return a handle to the dynamic prefab")
    }
}

/// The serialized representation of a prefab.
type DynamicPrefabData = Vec<HashMap<Uuid, ron::Value>>;

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn load_example() {
        let loader = PrefabLoader::new();
        loader.load("examples/assets/prefab/example.ron");
    }
}
