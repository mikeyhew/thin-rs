use super::ThinBackend;
use super::{DynSized, AssembleSafe};
use super::FnMove;
use dyn_sized::{self};

use std::ptr::{self, Unique};
use std::marker::{Unsize, PhantomData};
use std::mem;
use std::ops::{Deref, DerefMut};
use std::slice;
use alloc::heap::{Heap, Alloc, Layout};

/// A thin version of `Box`.
pub struct ThinBox<D> where
    D: DynSized + ?Sized
{
    backend_ptr: Unique<()>,
    _marker: PhantomData<D>
}

impl<D> Drop for ThinBox<D> where
    D: DynSized + ?Sized
{
    fn drop(&mut self) {
        unsafe {
            ptr::read(self).into_box();
        }
    }
}

impl<D> ThinBox<D> where
    D: DynSized + ?Sized
{
    pub fn new<S: Unsize<D>>(value: S) -> ThinBox<D> {
        let backend = ThinBackend::new(value);
        let bx = Box::new(backend) as Box<ThinBackend<D, D>>;
        ThinBox::from_box(bx)
    }
    
    /// performs a shallow drop, freeing the memory owned by `self` without
    /// dropping the contained value
    pub fn free(self) {
        free(self.into_box());
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
            let ptr = ThinBackend::assemble_mut((), self.backend_ptr.as_ptr());
            let bx = Box::from_raw(ptr);

            // this call to mem::forget is critical. I forgot to call it, and it cost me hours of debugging
            // infinite recursion in Drop
            mem::forget(self);
            bx
        }
    }

    pub fn into_boxed_value(self) -> Box<D> {
        let new_box = unsafe { copy_into_new_box(&*self) };
        free(self.into_box());
        new_box
    }

    fn as_ptr(&self) -> *const D {
        unsafe {
            let backend_fat = ThinBackend::assemble((), self.backend_ptr.as_ptr());
            &(**backend_fat) as *const D
        }
    }

    fn as_mut_ptr(&mut self) -> *mut D {
        unsafe {
            let backend_fat = ThinBackend::assemble_mut((), self.backend_ptr.as_ptr());
            &mut **backend_fat as *mut D
        }
    }
}

impl<D> ThinBox<D> where
    D: AssembleSafe + ?Sized
{
    pub unsafe fn copy_into_new(src: &D) -> ThinBox<D> {
        let size = <ThinBackend<D,D>>::size_of_backend(src);
        let align = <ThinBackend<D,D>>::align_of_backend(src);

        let layout = Layout::from_size_align_unchecked(size, align);

        let new_data_ptr: *mut () = if size == 0 {
            Unique::<()>::empty().as_ptr()
        } else {
            match Heap.alloc(layout) {
                Ok(ptr) => ptr as *mut (),
                Err(err) => Heap.oom(err)
            }
        };
        
        let backend_ptr = new_data_ptr as *mut ThinBackend<D, ()>;
        ptr::write(&mut (*backend_ptr).meta, src.meta());
        let backend_ptr: *mut ThinBackend<D,D> = ThinBackend::assemble_mut((), backend_ptr as *mut ());

        ptr::copy_nonoverlapping(
            src as *const D as *const u8,
            &mut (*backend_ptr).value as *mut D as *mut u8,
            dyn_sized::size_of_val::<D>(src.meta())
        );

        ThinBox {
            backend_ptr: Unique::new(backend_ptr as *mut ()).unwrap(),
            _marker: PhantomData
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

impl<F, Args> FnOnce<Args> for ThinBox<F> where
    F: FnMove<Args> + DynSized + ?Sized
{
    type Output = F::Output;

    extern "rust-call" fn call_once(mut self, args: Args) -> F::Output {
        unsafe {
            let ret = (&mut *self).call_move(args);
            free(self.into_box());
            ret
        }
    }
}

/// frees the `Box`'s memory, but does not drop the value it contains
fn free<T: ?Sized> (bx: Box<T>) {
    let size = mem::size_of_val(&*bx);
    let ptr = Box::into_raw(bx);
    unsafe {
        let slice: &mut [u8] = slice::from_raw_parts_mut(ptr as *mut u8, size);
        Box::from_raw(slice);
    }
}

unsafe fn copy_into_new_box<D: DynSized + ?Sized>(src: &D) -> Box<D> {
    let layout = Layout::for_value::<D>(&*src);
    let value_size = mem::size_of_val(&*src);

    let dest = if value_size == 0 {
        Unique::<()>::empty().as_ptr()
    } else {
        match Heap.alloc(layout) {
            Ok(ptr) => ptr as *mut (),
            Err(err) => Heap.oom(err)
        }
    };
    
    ptr::copy_nonoverlapping(
        src as *const D as *const u8,
        dest as *mut u8,
        value_size
    );

    Box::from_raw(D::assemble_mut(src.meta(), dest))
}

#[test]
fn test_copy_into_new_box() {
    let bx = Box::new([1,2,3]) as Box<[i32]>;
    let new_box: Box<[i32]> = unsafe { copy_into_new_box(&*bx) };
    assert_eq!(bx, new_box);
}

#[test]
fn test_box_free() {
    static mut FOO_DROP_COUNT: i32 = 0;
    unsafe { FOO_DROP_COUNT = 0; }
    struct Foo;
    impl Drop for Foo {
        fn drop(&mut self) {
            unsafe { FOO_DROP_COUNT += 1 };
        }
    }
    use std::any::Any;
    use dyn_sized::WrapSized;
    let bx = Box::new(WrapSized(Foo)) as Box<Any>;
    
    // move *bx into bx2
    let bx2 = unsafe { copy_into_new_box(&*bx) };
    free(bx);

    assert_eq!(0, unsafe { FOO_DROP_COUNT });
    mem::drop(bx2);
    assert_eq!(1, unsafe { FOO_DROP_COUNT });
}

#[test]
fn test_thin_box() {

    let tb: ThinBox<[i32]> = ThinBox::new([1,2,3]);

    trait Foo {}
    derive_DynSized!(Foo);

    assert!(mem::size_of::<ThinBox<Foo>>() < mem::size_of::<Box<Foo>>());

    let tb_size = mem::size_of_val(&*tb);

    let bx = tb.into_box();
    let bx_size = mem::size_of_val(&**bx);

    assert_eq!(tb_size, bx_size);

    assert_eq!(&**bx, &[1,2,3]);
}
