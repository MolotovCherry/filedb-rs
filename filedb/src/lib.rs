//mod file_types;
mod filedb;
mod type_uuid;

pub mod file_schema_capnp {
    include!(concat!(env!("OUT_DIR"), "/schema/file_schema_capnp.rs"));
}

pub use filedb::FileDb;
pub use filedb_uuid::TypeUuid;
pub use type_uuid::{TypeUuid, Uuid};
