use std::cell::RefCell;
use std::fmt;
use std::fmt::Formatter;
use std::rc::Rc;

// Re-exports
pub use std::cell::{Ref, RefMut};
pub use std::rc::Weak as RcWeak;

/// A shared, mutable handle.
pub struct Handle<T> {
    rc: Rc<RefCell<T>>,
}

impl<T> Handle<T> {
    pub fn new(value: T) -> Self {
        Self {
            rc: Rc::new(RefCell::new(value)),
        }
    }

    pub fn into_inner(self) -> T {
        let Self { rc } = self;
        match Rc::try_unwrap(rc) {
            Err(_) => panic!("handle is not unique"),
            Ok(ref_cell) => ref_cell.into_inner(),
        }
    }

    #[inline(always)]
    pub fn borrow(&self) -> Ref<'_, T> {
        self.rc.borrow()
    }

    #[inline(always)]
    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.rc.borrow_mut()
    }

    pub fn ptr_eq(&self, other: &Handle<T>) -> bool {
        Rc::ptr_eq(&self.rc, &other.rc)
    }

    /// TODO: Weak newtype so users can omit `RefCell` from `Weak<RefCell<...>>`
    pub fn downgrade(&self) -> RcWeak<RefCell<T>> {
        Rc::downgrade(&self.rc)
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Handle {
            rc: self.rc.clone(),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Handle").field(&*self.rc.borrow()).finish()
    }
}

/// A [`Handle`] shared in a circular reference.
pub enum Shared<T> {
    Strong(Handle<T>),
    Weak(RcWeak<RefCell<T>>),
}

impl<T> Shared<T> {
    pub fn strong(&self) -> Option<&Handle<T>> {
        match self {
            Shared::Strong(handle) => Some(handle),
            Shared::Weak(_) => None,
        }
    }

    pub fn upgrade(&self) -> Option<Handle<T>> {
        match self {
            Shared::Strong(handle) => Some(handle.clone()),
            Shared::Weak(weak) => weak.upgrade().map(|rc| Handle { rc }),
        }
    }

    pub fn weak(&self) -> Option<&RcWeak<RefCell<T>>> {
        match self {
            Shared::Strong(_) => None,
            Shared::Weak(weak) => Some(weak),
        }
    }

    pub fn downgrade(&self) -> RcWeak<RefCell<T>> {
        match self {
            Shared::Strong(handle) => handle.downgrade(),
            Shared::Weak(weak) => weak.clone(),
        }
    }
}
