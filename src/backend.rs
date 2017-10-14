use super::dyn_sized::{DynSized, AssembleSafe};

use std::ptr;
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

impl<D> DynSized for ThinBackend<D, D> where
    D: DynSized + ?Sized
{
    type Meta = ();

    unsafe fn assemble(_: (), data: *const ()) -> *const Self {
        let meta = ptr::read(data as *const ThinBackend<D, ()>).meta;
        mem::transmute(D::assemble(meta, data))
    }

    fn disassemble(ptr: *const Self) -> ((), *const ()) {
        ((), ptr as *const ())
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
    D: AssembleSafe + ?Sized
{
    pub fn size_of_backend(src: &D) -> usize {

        let src_as_backend: &ThinBackend<D, D> = unsafe {
            mem::transmute(src)
        };

        mem::size_of_val(src_as_backend)
    }

    pub fn align_of_backend(src: &D) -> usize {

        let src_as_backend: &ThinBackend<D, D> = unsafe {
            mem::transmute(src)
        };

        mem::align_of_val(src_as_backend)
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
