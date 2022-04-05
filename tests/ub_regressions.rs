use selfie::refs::{Mut, Ref};
use selfie::{Selfie, SelfieMut};
use std::cell::Cell;
use std::pin::Pin;

// From https://github.com/Kimundi/owning-ref-rs/issues/49

fn helper(owning_ref: Selfie<Box<Cell<u8>>, Ref<Cell<u8>>>) -> u8 {
    owning_ref.owned().set(10);
    owning_ref.with_referential(|r| r.set(20));
    owning_ref.owned().get()
}

#[test]
fn cell() {
    let val = Box::pin(Cell::new(25));
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
    let val = Box::pin(Cell::new(25));
    let owning_ref = SelfieMut::new(val, |c| Pin::get_mut(c));
    let res = helper_mut(owning_ref);
    assert_eq!(res, 20);
}

// From https://github.com/SabrinaJewson/pinned-aliasable : this detects miscompilation by the Rust compiler

struct Helper {
    reference: &'static Cell<u8>,
    owner: Box<Cell<u8>>,
}

fn helper_x(x: Helper) -> u8 {
    x.owner.set(10);
    x.reference.set(20);
    x.owner.get()
}

#[test]
fn miscompile() {
    let owner = Box::new(Cell::new(0));
    let reference = unsafe { &*(&*owner as *const Cell<u8>) };
    let x = Helper { reference, owner };
    assert_eq!(20, helper_x(x));
}
