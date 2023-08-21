use serde::{Deserialize, Serialize};

/// A uuid used for representing something unique
#[derive(Debug, Serialize, Deserialize)]
pub struct Uuid(pub u64);

/// TypeId used to differentiate between different types
///
/// This is unsafe because we rely on this to make sure the
/// deserialized type's data properly matches.
///
/// # Safety
///
/// User must ensure that each type must have a unique uuid.
/// If anything changes in that type, whether it be fields,
/// types used in fields, size, etc, you must generate a new unique
/// uuid that has never been used before
pub unsafe trait TypeUuid {
    fn uuid() -> Uuid;
}
