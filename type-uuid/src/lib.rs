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
    const UUID: u128;
}

/// Allows the TypeUuid constants to be retrieved via a trait object.  It is automatically implemented
/// for all types that implement TypeUuid.
///
/// It is theoretically possible to manually implement this independent of `TypeUuid`.  Please don't.
/// It is critical that this return value be deterministic, and manual implementation could prevent that.
pub trait TypeUuidDynamic {
    fn uuid(&self) -> u128;
}

impl<T: TypeUuid> TypeUuidDynamic for T {
    fn uuid(&self) -> u128 {
        Self::UUID
    }
}

impl TypeUuid for () {
    const UUID: u128 = 23818894022279401834075037072386988352;
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn type_uuid_trait_object() {
        let trait_object = Box::new(()) as Box<TypeUuidDynamic>;
        println!("UUID for (): {:#x}", trait_object.uuid());
    }
}
