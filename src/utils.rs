use core::pin::Pin;

pub unsafe fn transmute_pin<T: ?Sized>(pin: Pin<&T>) -> Pin<&'static T> {
    ::core::mem::transmute(pin)
}

pub unsafe fn transmute_pin_mut<T: ?Sized>(pin: Pin<&mut T>) -> Pin<&'static mut T> {
    ::core::mem::transmute(pin)
}
