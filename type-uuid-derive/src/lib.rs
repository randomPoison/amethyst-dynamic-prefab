extern crate proc_macro;

use darling::*;
use proc_macro::*;
use quote::quote;
use syn;
use uuid::Uuid;

#[proc_macro_derive(TypeUuid, attributes(uuid))]
pub fn hello_macro_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_hello_macro(&ast)
}

fn impl_hello_macro(ast: &syn::DeriveInput) -> TokenStream {
    let opts = MyTraitOpts::from_derive_input(ast).expect("Invalid derive input");

    let name = &opts.ident;

    let uuid = Uuid::parse_str(&opts.uuid.uuid_str).expect("Invalid UUID string");
    let bytes = uuid.as_bytes().iter().map(|byte| format!("{:#X}", byte));

    let gen = quote! {
        impl TypeUuid for #name {
            const UUID: uuid::Bytes = [
                #( #bytes ),*
            ];
        }
    };
    gen.into()
}

#[derive(FromDeriveInput)]
#[darling(attributes(uuid))]
struct MyTraitOpts {
    ident: syn::Ident,
    uuid: UuidAttr,
}

#[derive(Default, FromMeta)]
struct UuidAttr {
    uuid_str: String,
}
