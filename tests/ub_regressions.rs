use selfie::refs::{Mut, Ref};
use selfie::{Selfie, SelfieMut};
use std::cell::Cell;

// From https://github.com/Kimundi/owning-ref-rs/issues/49

fn helper(owning_ref: Selfie<Box<Cell<u8>>, Ref<Cell<u8>>>) -> u8 {
    owning_ref.owned().set(10);
    owning_ref.with_referential(|r| r.set(20));
    owning_ref.owned().get()
}

#[test]
fn cell() {
    let val = Box::new(Cell::new(25));
    let owning_ref = Selfie::new(val, |c| c);
    let res = helper(owning_ref);
    assert_eq!(res, 20);
}

fn helper_mut(owning_ref: SelfieMut<Box<Cell<u8>>, Mut<Cell<u8>>>) -> u8 {
    owning_ref.with_referential(|r| r.set(20));
    owning_ref.with_referential(|r| r.get())
}

#[test]
fn cell_mut() {
    let val = Box::new(Cell::new(25));
    let owning_ref = SelfieMut::new(val, |c| c);
    let res = helper_mut(owning_ref);
    assert_eq!(res, 20);
}
