use selfie::refs::RefType;
use selfie::SelfieMut;
use std::cell::RefCell;

// From https://github.com/Voultapher/self_cell/issues/28

struct Bar<'a>(RefCell<(Option<&'a Bar<'a>>, String)>);

struct BarRef;
impl<'a> RefType<'a> for BarRef {
    type Ref = Bar<'a>;
}

fn main() {
    let value = Box::pin(());

    let mut selfie: SelfieMut<Box<()>, BarRef> =
        SelfieMut::new(value, |_| Bar(RefCell::new((None, "Hello".to_owned()))));

    selfie.referential().0.borrow_mut().0 = Some(selfie.referential());

    let dep = selfie.referential_mut();
    let r1 = dep.0.get_mut();
    let string_ref_1 = &mut r1.1;
    let mut r2 = r1.0.unwrap().0.borrow_mut();
    let string_ref_2 = &mut r2.1;

    let s = &string_ref_1[..];
    string_ref_2.clear();
    string_ref_2.shrink_to_fit();
    println!("{}", s); // prints garbage
}
