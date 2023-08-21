#![allow(unused)]

use crc32fast::Hasher;
use flate2::bufread::DeflateDecoder;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, SeekFrom, Write};
#[cfg(target_os = "linux")]
use std::os::unix::fs::FileExt;
#[cfg(target_os = "windows")]
use std::os::windows::fs::FileExt;
use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};
use std::{
    fs::{File, OpenOptions},
    path::PathBuf,
};
use thiserror::Error;

use crate::file_schema_capnp::*;

// #[derive(Debug, Error)]
// pub enum FileDbError {
//     #[error("Failed to serialize data: {0:?}")]
//     IoError(#[from] std::io::Error),
//     #[error("Failed to serialize/deserialize data: {0:?}")]
//     SerDerializationFailed(#[from] bincode::Error),
//     #[error("T is invalid for the data stored in this key")]
//     InvalidType,
//     #[error("Invalid data type for key")]
//     InvalidDataType,
//     #[error("Key does not exist")]
//     KeyNotFound,
//     #[error("Header is corrupted")]
//     CorruptedHeader,
//     #[error("crc failed verification")]
//     DataIntegrityFailure,
// }

// // The maximum allowed size per chunk in bytes
// // maximum allowed bytes is VarType::MAX
// type VarType = u32;
// const SIZEOF_VAR: usize = std::mem::size_of::<VarType>();

// #[derive(Debug)]
// pub struct FileDbReader {
//     file: File,
//     offset: u32,
//     file_metadata: FileMetadata,
// }

// impl FileDbReader {
//     /// If you get ANY errors or weird issues, such as failed allocations, it means you are trying to deserialize the wrong type
//     pub fn read_data<T: DeserializeOwned>(&self, name: &str) -> Result<T, FileDbError> {
//         let datachunk = self
//             .file_metadata
//             .index_map
//             .get(name)
//             .ok_or(FileDbError::KeyNotFound)?;

//         let offset = self.offset as u64 + datachunk.location;

//         // could really use a way to verify the typeid, but rust's typeid isn't guaranteed across rust releases

//         // this is pretty obvious - the key needs to be the right datatype to access it
//         // if datachunk.data_type != DType::VAR {
//         //     return Err(FileDbError::InvalidDataType);
//         // }

//         // T must be the same size as the data chunk, or the layout isn't even the same
//         if std::mem::size_of::<T>() as u64
//             != datachunk.size_of_t.ok_or(FileDbError::CorruptedHeader)?
//         {
//             return Err(FileDbError::InvalidType);
//         }

//         // data
//         let mut data_bytes = vec![0; datachunk.length as usize];

//         #[cfg(target_os = "windows")]
//         self.file.seek_read(&mut data_bytes, offset)?;
//         #[cfg(target_os = "linux")]
//         self.file.read_exact_at(&mut data_bytes, offset)?;

//         // perform crc analysis again and see if it matches
//         let mut hasher = Hasher::new();
//         hasher.update(&data_bytes);
//         let verify_crc = hasher.finalize();

//         // verify crc
//         if (datachunk.crc != verify_crc) {
//             return Err(FileDbError::DataIntegrityFailure);
//         }

//         let reader = DeflateDecoder::new(&data_bytes[..]);
//         let data = bincode::deserialize_from::<_, T>(reader)?;

//         Ok(data)
//     }
// }

/*
#[derive(Debug)]
pub struct FileDb {
    magic_header: String,
    filepath: PathBuf,
    file_metadata: FileMetadata,
    // data to insert into file
    buffer: BTreeMap<String, Vec<u8>>,
}

impl FileDb {
    pub fn new<S: Into<String>, P: Into<PathBuf>>(
        magic_header: S,
        filepath: P,
        version: u32,
    ) -> Self {
        let metadata = FileMetadata {
            version,
            ..Default::default()
        };

        let magic_header = magic_header.into();

        // we allow a pretty big size magic header, but please control yourself!
        assert!(magic_header.len() <= u8::MAX as usize);

        Self {
            magic_header,
            filepath: filepath.into(),
            file_metadata: metadata,
            buffer: BTreeMap::default(),
        }
    }

    pub fn open(filename: &str) -> Result<FileDbReader, FileDbError> {
        let mut options = OpenOptions::new();
        let mut file = options.read(true).open(filename)?;

        file.seek(SeekFrom::Start(Self::MAGIC_HEADER_LEN as u64))?;

        // data length
        let mut header_len = [0; SIZEOF_VAR];
        file.read_exact(&mut header_len)?;
        let header_len: VarType = bytemuck::cast(header_len);

        // crc
        let mut data_crc = [0; std::mem::size_of::<u32>()];
        file.read_exact(&mut data_crc)?;
        let data_crc: u32 = bytemuck::cast(data_crc);

        // data
        let mut header_bytes = vec![0; header_len as usize];
        file.read_exact(&mut header_bytes)?;

        // perform crc analysis again and see if it matches
        let mut hasher = Hasher::new();
        hasher.update(&header_bytes);
        let verify_crc = hasher.finalize();

        // verify crc
        if (data_crc != verify_crc) {
            return Err(FileDbError::DataIntegrityFailure);
        }

        let reader = DeflateDecoder::new(&header_bytes[..]);
        let file_metadata = bincode::deserialize_from::<_, FileMetadata>(reader)?;

        Ok(FileDbReader {
            file,
            file_metadata,
            offset:
                // header length
                Self::MAGIC_HEADER_LEN as u32
                // data length
                + SIZEOF_VAR as u32
                // crc
                + std::mem::size_of::<u32>() as u32
                // header data
                + header_len,
        })
    }

    pub fn insert_metadata(&mut self, name: &str, val: &str) -> &mut Self {
        self.file_metadata
            .metadata
            .get_or_insert_with(HashMap::new)
            .insert(name.into(), val.into());

        self
    }

    /// Insert a piece of data into the file with
    pub fn insert_data<Key, Data>(&mut self, name: Key, data: &Data) -> Result<(), FileDbError>
    where
        Key: Serialize,
        Data: Serialize + TypeUuid + Hash,
    {
        let mut writer = DeflateEncoder::new(Vec::new(), Compression::default());
        bincode::serialize_into(&mut writer, data)?;
        let data = writer.finish()?;

        assert!(data.len() <= VarType::MAX as usize);

        let mut hasher = Hasher::new();
        hasher.update(&data);
        let crc = hasher.finalize();

        let data_chunk = DataChunk {
            length: data.len() as u64,
            crc,
            size_of_t: Some(std::mem::size_of::<Data>() as u64),
            ..Default::default()
        };

        self.file_metadata
            .index_map
            .insert(name.to_string(), data_chunk);

        self.buffer.insert(name.to_string(), data);

        Ok(())
    }

    pub fn finish(mut self) -> Result<(), FileDbError> {
        let mut options = OpenOptions::new();
        let mut file = options.write(true).create(true).open(&self.filepath)?;

        file.set_len(0)?;
        file.seek(SeekFrom::End(0))?;

        // magic header
        let mut buffer = [0u8; Self::MAGIC_HEADER_LEN];
        let mut cursor = std::io::Cursor::new(&mut buffer[..]);
        cursor.write_all(self.magic_header.as_bytes()).unwrap();

        // counter to keep track of location of chunks
        let mut location = 0u64;
        // first insert the keys to get the size of the original header
        for (key, data) in self.buffer.iter() {
            let chunk = self.file_metadata.index_map.get_mut(key).unwrap();
            chunk.location = location;
            location += chunk.length;
        }

        let mut writer = DeflateEncoder::new(Vec::new(), Compression::default());
        bincode::serialize_into(&mut writer, &self.file_metadata)?;
        let header = writer.finish()?;

        assert!(header.len() <= VarType::MAX as usize);

        // write magic header
        file.write_all(&buffer)?;

        // write header len after
        let header_len: [u8; SIZEOF_VAR] = bytemuck::cast(header.len() as VarType);
        file.write_all(&header_len)?;

        // now write the crc
        let mut hasher = Hasher::new();
        hasher.update(&header);
        let crc = hasher.finalize();

        let crc: [u8; SIZEOF_VAR] = bytemuck::cast(crc);
        file.write_all(&crc)?;

        // and finally write the header data
        file.write_all(&header)?;

        for (_, data) in self.buffer {
            // finally write data
            file.write_all(&data)?;
        }

        Ok(())
    }
}
*/

pub struct FileDb<'a> {
    magic_number: &'a [u8],
}

impl<'a> FileDb<'a> {
    pub fn new(magic_number: &'a [u8]) -> Self {
        Self { magic_number }
    }

    pub fn finish(self) -> Vec<u8> {
        let mut message = ::capnp::message::Builder::new_default();
        let mut header_b = message.init_root::<header::Builder>();

        header_b.set_version(VERSION);
        capnp::serialize::write_message_to_words(&message)
    }
}
