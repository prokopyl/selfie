use core::marker::PhantomData;

pub trait RefType<'a> {
    type Ref: 'a + Sized;
}

pub struct Ref<T: ?Sized>(PhantomData<T>);

// TODO: 'static
impl<'a, T: 'a + ?Sized> RefType<'a> for Ref<T> {
    type Ref = &'a T;
}

pub struct Mut<T: ?Sized>(PhantomData<T>);

// TODO: 'static
impl<'a, T: 'a + ?Sized> RefType<'a> for Mut<T> {
    type Ref = &'a mut T;
}
