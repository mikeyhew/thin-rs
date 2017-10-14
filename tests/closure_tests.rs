
extern crate thin;
extern crate fn_move;

use thin::ThinBox;
use thin::ThinBackend;
use fn_move::FnMove;

#[test]
fn thin_box_closure() {

    let v1 = vec![1i32,2,3];
    let v2 = vec![-1i32, -2, -3];

    let closure: ThinBox<FnMove() -> Vec<i32>> = ThinBox::new(move ||{
        v1.into_iter()
        .zip(v2)
        // have to return a Vec because fixed-size arrays still
        // don't implement IntoIterator
        .flat_map(|(x, y)| { vec![x, y] })
        .collect::<Vec<_>>()
    });

    closure();

    // assert_eq!(closure(), &[1, -1, 2, -2, 3, -3]);
}
