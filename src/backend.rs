use super::dyn_sized::{self, DynSized};

use std::marker::Unsize;
use std::mem;
use std::ops::{Deref, DerefMut};

/// The storage format for thin pointers. Stores the metadata and the value together.
#[derive(Debug)]
pub struct ThinBackend<D, T> where
    D: DynSized + ?Sized,
    T: ?Sized
{
    pub meta: D::Meta,
    pub value: T
}

unsafe impl<D> DynSized for ThinBackend<D, D> where
    D: DynSized + ?Sized
{
    type Meta = D::Meta;

    fn assemble(meta: D::Meta, data: *const ()) -> *const Self {
        let d_ptr: *const D = D::assemble(meta, data);
        unsafe {
            mem::transmute(d_ptr)
        }
    }

    fn disassemble(ptr: *const Self) -> (D::Meta, *const ()) {
        let d_ptr: *const D = unsafe {
            mem::transmute(ptr)
        };
        D::disassemble(d_ptr)
    }
}

impl<D, S> ThinBackend<D, S> where
    D: DynSized + ?Sized,
    S: Unsize<D>
{
    pub fn new(value: S) -> ThinBackend<D, S> {
        ThinBackend {
            meta: (&value as &D).meta(),
            value: value
        }
    }
}

impl<D, S> ThinBackend<D, S> where
    D: DynSized + ?Sized
{
    pub fn into_value(self) -> S {
        self.value
    }
}

impl<D> ThinBackend<D, D> where
    D: DynSized + ?Sized
{
    pub unsafe fn fat_from_thin(data: *const ()) -> *const ThinBackend<D, D> {
        let ptr = data as *const ThinBackend<D, ()>;
        let meta = (*ptr).meta;
        ThinBackend::assemble(meta, data)
    }

    pub unsafe fn fat_from_thin_mut(data: *mut ()) -> *mut ThinBackend<D, D> {
        let ptr = data as *mut ThinBackend<D, ()>;
        let meta = (*ptr).meta;
        ThinBackend::assemble_mut(meta, data)
    }

    pub fn size_of_backend(src: &D) -> usize {
        dyn_sized::size_of_val::<ThinBackend<D, D>>(src.meta())
    }

    pub fn align_of_backend(src: &D) -> usize {
        dyn_sized::align_of_val::<ThinBackend<D,D>>(src.meta())
    }
}

impl<D, T> Deref for ThinBackend<D, T> where
    D: DynSized + ?Sized,
    T: ?Sized
{
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<D, T> DerefMut for ThinBackend<D, T> where
    D: DynSized + ?Sized,
    T: ?Sized
{
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}
