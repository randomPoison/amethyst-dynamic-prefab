use crate::*;
use serde::de::DeserializeOwned;
use serde::*;

impl TypeUuid for amethyst::core::Transform {
    const UUID: u128 = 0xf3d49cc2c77e4dc99e1fc01e9279c999;
}

impl TypeUuid for amethyst::renderer::CameraPrefab {
    const UUID: u128 = 0x15ed1b66537e4e75b52dcd4659ba53bf;
}

impl TypeUuid for amethyst::renderer::LightPrefab {
    const UUID: u128 = 0x41c40489269b4ef5af7f675a29473f86;
}

impl<V, M, T> TypeUuid for amethyst::renderer::GraphicsPrefab<V, M, T>
where
    // Require that all the parameters implement `TypeUuid` so that we can compose
    // a new UUID based on the concrete parameters.
    V: TypeUuid,
    M: TypeUuid,
    T: TypeUuid,

    // The built-in constraints as declared by `GraphicsPrefab<V, M, T>`.
    M: amethyst::assets::Format<amethyst::renderer::Mesh>,
    M::Options: DeserializeOwned + Serialize,
    T: amethyst::assets::Format<
        amethyst::renderer::Texture,
        Options = amethyst::renderer::TextureMetadata,
    >,
{
    const UUID: u128 = compose_uuid(
        0xd29ba396a10448fea14085b9a489e26d,
        compose_uuid(V::UUID, compose_uuid(M::UUID, T::UUID)),
    );
}

// TODO: THIS IS TERRIBLE OMG PLEASE FIX THIS.
const fn compose_uuid(left: u128, right: u128) -> u128 {
    left ^ right
}
