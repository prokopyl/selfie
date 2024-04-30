use alloc::boxed::Box;
use core::ops::Coroutine;
/*
struct Foo<C: ?Sized> {
    x: u8,
    coro: C,
}

impl<C: ?Sized> Foo<C> {
    pub fn new() -> Self {
        todo!()
    }
}

fn foo_coro(
    coro: impl Coroutine<(), Return = (), Yield = ()> + 'static,
) -> Box<Foo<dyn Coroutine<(), Return = (), Yield = ()>>> {
    Box::new(Foo { x: 42, coro })
}
*/

#[cfg(test)]
mod test {
    use alloc::string::String;
    use core::ops::{Coroutine, CoroutineState};
    use core::pin::Pin;

    #[test]
    pub fn test() {
        let strtest = String::from("hello");
        let mut coroutine = move || {
            let refstr = &strtest[0..1];

            let x = 5;
            //yield &refstr as *const _;
            dbg!(&x);
        };

        dbg!(core::mem::size_of_val(&coroutine));
    }

    async fn foo(str: &str) {
        todo!()
    }

    #[test]
    pub fn test_async() {
        let strtest = String::from("hello");
        let mut coroutine = move || async {
            let strtest = strtest;
            let refstr = &strtest[0..1];

            let x = 5;
            foo(&refstr).await;
            dbg!(&x);
        };

        dbg!(core::mem::size_of_val(&coroutine));
    }
}
