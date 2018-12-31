#[cfg(feature = "amethyst")]
mod amethyst_types;

/// Provides a statically defined UUID for a Rust type.  It's recommended to implement this
/// by generating a v4 UUID, and transmuting it into a `u128`.  Here's an example of how to do so
///
/// ```
/// extern crate uuid;
/// use std::mem::transmute;
/// use uuid::Uuid;
///
/// fn main() {
///     println!("{}", unsafe {transmute::<[u8; 16], u128>(*Uuid::new_v4().as_bytes())});
/// }
/// ```
pub trait TypeUuid {
    const UUID: uuid::Bytes;
}

/// Allows the TypeUuid constants to be retrieved via a trait object.  It is automatically implemented
/// for all types that implement TypeUuid.
///
/// It is theoretically possible to manually implement this independent of `TypeUuid`.  Please don't.
/// It is critical that this return value be deterministic, and manual implementation could prevent that.
pub trait TypeUuidDynamic {
    fn uuid(&self) -> uuid::Bytes;
}

impl<T: TypeUuid> TypeUuidDynamic for T {
    fn uuid(&self) -> uuid::Bytes {
        Self::UUID
    }
}

impl TypeUuid for () {
    const UUID: uuid::Bytes = [
        0x98, 0xF1, 0x8B, 0x7E, 0x4E, 0xB9, 0x42, 0x9C, 0xAF, 0xBF, 0xEE, 0x2F, 0x9F, 0x4C, 0xBC,
        0x7,
    ];
}

#[cfg(test)]
mod test {
    use crate::*;

    /// Verifies that `TypeUuidDynamic` can be instantiated as a trait object.
    #[test]
    fn type_uuid_trait_object() {
        let trait_object = Box::new(()) as Box<TypeUuidDynamic>;
        println!("UUID for (): {:#X?}", trait_object.uuid());
    }
}
