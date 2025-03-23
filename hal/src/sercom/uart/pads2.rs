
use crate::sercom::*;
use crate::gpio::*;
use crate::typelevel::NoneT;

trait Rxpo {}

struct Rx {}
struct Tx {}
struct Clk {}
struct Rts {}
struct Cts {}
struct Unused {}

trait UartRole {}
impl UartRole for Rx {}
impl UartRole for Tx {}
impl UartRole for Clk {}
impl UartRole for Rts {}
impl UartRole for Cts {}
impl UartRole for Unused {}

trait UartPin {
	type Role: UartRole;
	type Pin: SomePin;
}

pub struct Pads<S: Sercom, P0: UartPin<Pin: OptionalPin<Id: GetPad<S, PadNum = Pad0>>>, P1: UartPin<Pin: OptionalPin<Id: GetPad<S, PadNum = Pad1>>>> {
	s: S,
	p0: P0,
	p1: P1,
}


