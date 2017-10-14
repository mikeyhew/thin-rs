#![feature(unique, unsize, fnbox, fn_traits, unboxed_closures, alloc, heap_api, allocator_api, const_cell_new)]

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
