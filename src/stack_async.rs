use alloc::string::String;
use core::fmt::Display;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};

struct NeverFuture<'a>(PhantomData<&'a ()>);

impl<'a> Future for NeverFuture<'a> {
    type Output = ();

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}

async fn foo(_: ()) {}

const fn size_of_return_val<P, T>(originator: fn(P) -> T) -> usize {
    core::mem::size_of::<T>()
}

type Ret<F, X>
where
    F: FnOnce() -> X,
= (X, PhantomData<F>);

trait GetOutput {
    type Output: Sized;
}

impl<I, O> GetOutput for fn(I) -> O {
    type Output = O;
}

#[cfg(test)]
mod test {
    use crate::stack_async::{foo, size_of_return_val};
    use std::prelude::v1::String;

    #[repr(align(4096))]
    struct StupidlyAligned;
    struct WrapAligned(String, StupidlyAligned);

    #[test]
    fn it_works() {
        let fut = size_of_return_val(foo);
        dbg!(fut);
        dbg!(core::mem::align_of::<[WrapAligned; 0]>());
    }
}
