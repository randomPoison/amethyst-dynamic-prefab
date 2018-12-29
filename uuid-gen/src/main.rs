extern crate uuid;
use uuid::Uuid;

fn main() {
    let uuid = Uuid::new_v4();
    println!("UUID: {}", uuid);
    println!("Hex: 0x{}", uuid.to_simple());
}
