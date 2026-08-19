//! Object header.

#[derive(Debug)]
pub enum ObjectKind {
    Environment,
}

pub struct ObjectHeader {
    kind: ObjectKind,
}
