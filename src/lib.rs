#![feature(unique, unsize, fnbox, fn_traits, unboxed_closures, alloc, heap_api, allocator_api)]
/*!
  Provides thin pointer types to dynamically sized types that implement the `DynSized` trait. By "thin", that means they store the metadata (i.e. slice length or vtable pointer) together with the data, instead of in the pointer itself. Currently provides `ThinBox`, a thin version of `Box`.
*/

#[allow(unused)]
#[macro_use]
pub extern crate dyn_sized;
extern crate fn_move;
extern crate alloc;

pub use dyn_sized::{DynSized, AssembleSafe};
pub use fn_move::FnMove;

mod backend;
pub use backend::ThinBackend;

mod boxed;
pub use boxed::ThinBox;
