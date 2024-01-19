use crate::refs::RefType;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};

pub struct SelfieMutAsyncCtx<R> {
    _foo: PhantomData<fn() -> R>,
}

impl<R> SelfieMutAsyncCtx<R> {
    pub fn new() -> Self {
        todo!()
    }
}

struct NeverFuture<'a>(PhantomData<&'a ()>);

impl<'a> Future for NeverFuture<'a> {
    type Output = ();

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}

impl<R> SelfieMutAsyncCtx<R> {
    pub fn send_referential<'a>(self, referential: &mut R::Ref) -> NeverFuture<'a>
    where
        R: RefType<'a>,
    {
        todo!();
        NeverFuture(PhantomData)
    }
}

pub async fn selfie_mut<O, R: for<'a> RefType<'a>>(
    ctx: SelfieMutAsyncCtx<R>,
    mut owned: O,
    make_referential: impl FnOnce(&mut O) -> <R as RefType<'_>>::Ref,
) -> O {
    let mut referential = make_referential(&mut owned);
    ctx.send_referential(&mut referential).await;

    drop(referential);

    owned
}

async fn foo(_: ()) -> u32 {
    NeverFuture(PhantomData).await;
    42
}

const fn transmute_fn<P, T>(originator: fn(P) -> T) -> T {
    todo!()
}

fn test() {
    let fut = transmute_fn(foo);
}

pub struct SelfieMut<'a, O, R>
where
    O: 'a,
    R: for<'this> RefType<'this>,
{
    _p: PhantomData<<R as RefType<'a>>::Ref>,
    future: dyn Future<Output = O>,
}
/*
impl<'a, O, R> SelfieMut<'a, O, R>
where
    O: 'a,
    R: for<'this> RefType<'this>,
{
    pub fn new(owned: O) -> Self {
        Self {
            _p: PhantomData,
            future: selfie_mut(SelfieMutAsyncCtx::new(), owned, || todo!()),
        }
    }
}
*/
