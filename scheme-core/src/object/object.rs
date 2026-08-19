//! Heap allocated object.
use crate::object::header::ObjectHeader;

/// Marker trait for structures that are allowed to be heap allocated.
pub trait HeapObject {}

/// A heap allocated object.
///
/// The `SchemeObject` struct is a wrapper around a heap allocated object, which
/// consists of an [`ObjectHeader`] and a value of type `T`. The `ObjectHeader`
/// contains metadata about the object, such as its type and size, while the
/// value is the actual data stored in the object.
///
/// The `SchemeObject` struct is used to represent objects in the Scheme interpreter, allowing for efficient memory management and garbage collection.
#[repr(C)]
pub struct SchemeObject<T: HeapObject + 'static> {
    header: ObjectHeader,
    value: T,
}

pub type ErasedObject = SchemeObject<Erased>;

#[doc(hidden)]
pub struct Erased {}

impl HeapObject for Erased {}
