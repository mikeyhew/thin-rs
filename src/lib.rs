#![feature(unique, unsize, fnbox, fn_traits, unboxed_closures)]

#[macro_use]
pub extern crate dyn_sized;

pub use dyn_sized::DynSized;
use dyn_sized::{WrapSized, PtrExt};

use std::ptr::{self, Unique};
use std::marker::{Unsize, PhantomData};
use std::mem;
use std::ops::{Deref, DerefMut};

pub struct ThinBackend<D, T> where
    D: DynSized + ?Sized,
    T: ?Sized
{
    meta: D::Meta,
    value: T
}

impl<D> DynSized for ThinBackend<D, D> where
    D: DynSized + ?Sized
{
    type Meta = ();

    unsafe fn assemble(_: (), data: *const ()) -> *const Self {
        let meta = ptr::read(data as *const ThinBackend<D, WrapSized<()>>).meta;
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

pub struct ThinBox<D> where
    D: DynSized + ?Sized
{
    backend_ptr: Unique<()>,
    _marker: PhantomData<D>
}

impl<D> ThinBox<D> where
    D: DynSized + ?Sized
{
    pub fn new<S: Unsize<D>>(value: S) -> ThinBox<D> {
        let backend = ThinBackend::new(value);
        let bx = Box::new(backend) as Box<ThinBackend<D, D>>;
        ThinBox::from_box(bx)
    }

    pub fn from_box(bx: Box<ThinBackend<D, D>>) -> ThinBox<D> {
        let ((), backend_ptr) = ThinBackend::disassemble_mut(Box::into_raw(bx));
        ThinBox {
            backend_ptr: unsafe { Unique::new_unchecked(backend_ptr) },
            _marker: PhantomData
        }
    }

    pub fn into_box(self) -> Box<ThinBackend<D, D>> {
        unsafe {
            Box::from_raw(ThinBackend::assemble_mut((), self.backend_ptr.as_ptr()))
        }
    }

    fn as_ptr(&self) -> *const D {
        unsafe {
            let backend_fat = ThinBackend::assemble((), self.backend_ptr.as_ptr());
            &(*backend_fat).value as *const D
        }
    }

    fn as_mut_ptr(&mut self) -> *mut D {
        unsafe {
            let backend_fat = ThinBackend::assemble_mut((), self.backend_ptr.as_ptr());
            &mut (*backend_fat).value as *mut D
        }
    }
}

impl<T> Deref for ThinBox<T> where
    T: DynSized + ?Sized
{
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {
            &*self.as_ptr()
        }
    }
}

impl<T> DerefMut for ThinBox<T> where
    T: DynSized + ?Sized
{
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            &mut *self.as_mut_ptr()
        }
    }
}

impl<T> AsRef<T> for ThinBox<T> where
    T: DynSized + ?Sized
{
    fn as_ref(&self) -> &T {
        &*self
    }
}

impl<T> AsMut<T> for ThinBox<T> where
    T: DynSized + ?Sized
{
    fn as_mut(&mut self) -> &mut T {
        &mut *self
    }
}

#[test]
fn test_thin_box() {

    let tb = ThinBox::new([1,2,3]) as ThinBox<[i32]>;

    trait Foo {}
    derive_DynSized!(Foo);

    assert!(mem::size_of::<ThinBox<Foo>>() < mem::size_of::<Box<Foo>>());

    let tb_size = mem::size_of_val(&*tb);

    let bx = tb.into_box();
    let bx_size = mem::size_of_val(&**bx);

    assert_eq!(tb_size, bx_size);

    assert_eq!(&**bx, &[1,2,3]);
}

#[test]
fn test_thin_box_closure() {
    pub trait FnOnceUnsafe<Args> {
        type Output;

        unsafe fn call_once_unsafe(&mut self, args: Args) -> Self::Output;
    }

    derive_DynSized!(FnOnceUnsafe<Args, Output=Output>, Args, Output);

    impl <F, Args, Output> FnOnceUnsafe<Args> for F where
        F: FnOnce<Args, Output=Output>
    {
        type Output = Output;

        unsafe fn call_once_unsafe(&mut self, args: Args) -> Output {
            ptr::read(self).call_once(args)
        }
    }

    impl<F, Args, Output> FnOnce<Args> for ThinBox<F> where
        F: FnOnceUnsafe<Args, Output=Output> + DynSized + ?Sized
    {
        type Output = Output;

        extern "rust-call" fn call_once(mut self, args: Args) -> Output {
            unsafe {
                self.call_once_unsafe(args)
            }
        }
    }

    let v = vec![1i32,2,3];
    let mut v2 = vec![];
    {
        let v2: &mut Vec<i32> = unsafe { &mut *(&mut v2 as *mut _)};

        let closure: ThinBox<FnOnceUnsafe()> = ThinBox::new(move || {
            for x in v {
                v2.push(x + 3);
            }
        });

        closure();
    }

    assert_eq!(v2, &[4,5,6])
}
