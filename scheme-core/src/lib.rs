//! Sceme core.

pub mod display;
pub mod error;
pub mod lexer;
mod object;
pub mod parser;
mod store;
mod symbol;
mod value;
mod vm;

pub use self::store::Store;

/// Convenience macro for declaring type safe identifiers.
///
/// ```
/// # use scheme_core::declare_id;
/// declare_id!(struct ConstantId(u16));
/// let func_id = ConstantId::new(42);
/// assert_eq!(func_id.as_inner(), 42);
/// ```
///
/// Supports a visibility modifier.
///
/// ```
/// # use scheme_core::declare_id;
/// declare_id!(pub(crate) struct LocalId(u8));
/// declare_id!(pub struct TypeId(u64));
/// # let id = LocalId::new(42);
/// # (id.as_inner(), 42);
/// # let id = TypeId::new(42);
/// # (id.as_inner(), 42);
/// ```
#[macro_export]
#[doc(hidden)]
macro_rules! declare_id {
    (
        $(#[$outer:meta])*
        $vis:vis struct $name:ident($ty:ty)
    ) => {
        $(#[$outer])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        #[repr(transparent)]
        $vis struct $name($ty);

        impl $name {
            #[inline]
            $vis const fn new(value: $ty) -> Self {
                Self(value)
            }

            #[inline]
            $vis const fn as_inner(self) -> $ty {
                self.0
            }

            #[inline]
            $vis const fn as_usize(self) -> usize {
                self.0 as usize
            }
        }

        impl From<$name> for usize {
            fn from(id: $name) -> usize {
                id.as_usize()
            }
        }
    };
}
