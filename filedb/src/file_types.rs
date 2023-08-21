use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::Uuid;

#[derive(Debug, Serialize, Deserialize, Default)]
pub(crate) struct Header {
    magic_number: Vec<u8>,
    /// used for keeping track of file's version
    /// can be used for compatibility
    version: u32,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum Block {
    Index(IndexBlock),
    Data(DataBlock),
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct IndexBlock {
    blocks_status: [BlockStatus; 30],
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct BlockStatus {
    status: BlockUseStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum BlockUseStatus {
    InUseBy(Uuid),
    Free,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct DataBlock {
    id: Uuid,
    // the type id for the data in this block
    type_id: Uuid,
    // the order of data to which this block belongs to
    order: u64,
    // how large the data chunk is - these vary in size depending on the data size
    length: u64,
    // a crc check for the data to ensure its integrity
    crc: u32,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub(crate) struct DataChunk<'a>(&'a [u8]);

impl<'a> Deref for DataChunk<'a> {
    type Target = &'a [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
