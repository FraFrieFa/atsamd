
use crate::sercom::*;
use crate::gpio::*;
use crate::typelevel::NoneT;
use core::marker::PhantomData;


trait UartPin {}
struct Rx<P: SomePin> { pin: P }
struct Tx<P: SomePin> { pin: P }
impl<P: SomePin> UartPin for Rx<P> {}
impl<P: SomePin> UartPin for Tx<P> {}

trait OptionalUartPin {}
impl<P: UartPin> OptionalUartPin for P {}
impl OptionalUartPin for NoneT {}

pub struct Pads<S: Sercom, P0: OptionalUartPin> {
	s: S,
	p0: P0,
}


fn test(s: Sercom1, pin: Pin<PA16, AlternateD>) {

	let p = Pads { s, p0: Rx { pin } };
	//let s = Pads { s, p0: NoneT };

}
