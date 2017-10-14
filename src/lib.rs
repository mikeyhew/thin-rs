#![feature(unique, unsize, fnbox, fn_traits, unboxed_closures, alloc, heap_api, allocator_api)]
/*!
Provides thin pointer types to dynamically sized types that implement the `DynSized` trait. By "thin", that means they store the metadata (i.e. slice length or vtable pointer) together with the data, instead of in the pointer itself. Currently provides `ThinBox`, a thin version of `Box`.

Example: storing a closure in a `ThinBox`;

```
extern crate thin;
#[macro_use] extern crate dyn_sized;
use thin::{ThinBox, FnMove};

// Define our own subtrait to get around the orphan rule for trait impls
// Have to use `FnMove` instead of `FnOnce` or `FnBox`
trait Closure: FnMove(&str) -> String {}
derive_DynSized!(Closure<Output=String>);

fn main() {
  let s1 = String::from("Hello");

  let closure: ThinBox<FnMove(&'static str) -> String> = ThinBox::new(move |s2|{
      s1 + " " + s2 + "!"
  });

  assert_eq!("Hello World!", closure("World"));
}
```

There's a couple things to notice here: one is that we needed to derive `DynSized` for our closure trait object type in order to store it in a `ThinBox`. And because of the orphan rule for trait impls, we needed to define our own trait `Closure` to do that. And the other thing to notice, is that our `Closure` trait has `FnMove` as a supertrait instead of `FnBox` or `FnOnce`, in order to be callable from inside a `ThinBox`. That's alright, though, since ThinBox<F: FnMove> implemnts FnOnce, and we're able to call the `ThinBox<Closure>` directly.
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
