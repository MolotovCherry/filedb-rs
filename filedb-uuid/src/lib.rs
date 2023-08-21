use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Create unique stable typeid for struct
///
/// Requirements:
/// - struct must stay the same size or uuid will change
/// - struct must textually remain the same. This means that even a type rename
///   will cause a uuid change, even though it is strictly speaking the same type
///
/// If you changed the name and require the same uuid to access your previous data,
/// you may manually implement [`TypeUuid`] to return the correct uuid, however be warned,
/// the type must be EXACTLY the same types and layout for it to be safe. It is recommended
/// to convert the data over to the new typeuuid's so that you do not need to custom
/// implement it anymore (as you mmust be diligent to change it to the correct uuid every time
/// anything changes)
///
/// There does exist a corner case where if the text content of the struct is the same, AND it's the
/// same size, you could replace one of the inner types with something of the same size/name and it will
/// still calculate the same uuid, but will be incorrect type
///
/// Unfortunately Rust doesn't allow us to truly generate a typeid that stays the same across compiles/crates
/// as long as it actually is the same, which is why we need this macro
#[proc_macro_derive(TypeUuid)]
pub fn derive_type_uuid(input: TokenStream) -> TokenStream {
    let tokens_str = input.to_string();

    let ast = parse_macro_input!(input as DeriveInput);
    let struct_ = ast.ident;

    let filedb = crate_name("filedb").expect("filedb crate is present in `Cargo.toml`");
    let filedb = match filedb {
        FoundCrate::Itself => unimplemented!(),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(#ident)
        }
    };

    // generate unique hash for token input
    let mut hasher = DefaultHasher::new();
    tokens_str.hash(&mut hasher);
    let hash = hasher.finish();

    quote! {
        unsafe impl #filedb::TypeUuid for #struct_ {
            fn uuid() -> #filedb::Uuid {
                const fn calc_uuid() -> u64 {
                    let hash = #hash;
                    // by converting size to `u8` we can ensure this always remains stable regardless
                    // of usize. At the time of writing, Rust does not have const stable number conversions,
                    // which would fix this issue.
                    // size must be <= 255 for this to work, since > 255 will all
                    // yield the same number and hash, causing false positives
                    let size = (std::mem::size_of::<#struct_>() as u8) as u64;
                    // hence, we do not support sizes > 255
                    // TODO: fix this once `const_num_from_num` is stable
                    //       https://github.com/rust-lang/rust/issues/87852
                    if size > u8::MAX as u64 {
                        panic!("struct is too big");
                    }

                    hash ^ size
                }

                const UUID: u64 = calc_uuid();
                #filedb::Uuid(UUID)
            }
        }
    }
    .into()
}
