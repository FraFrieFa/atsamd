
use crate::sercom::*;
use crate::gpio::*;
use crate::typelevel::NoneT;
use core::marker::PhantomData;

trait OptionalPadNum {}
impl<P: PadNum> OptionalPadNum for P {}
impl OptionalPadNum for NoneT {}

pub struct Pads<S: Sercom, P0: OptionalPin = NoneT, P1: OptionalPin = NoneT, P2: OptionalPin = NoneT, P3: OptionalPin = NoneT, Rx: OptionalPadNum = NoneT, Tx: OptionalPadNum = NoneT, Clk: OptionalPadNum = NoneT, Rts: OptionalPadNum = NoneT, Cts: OptionalPadNum = NoneT> {
	s: S,
	p0: P0,
	p1: P1,
	p2: P2,
	p3: P3,
	rx: PhantomData<Rx>,
	tx: PhantomData<Tx>,
	clk: PhantomData<Clk>,
	rts: PhantomData<Rts>,
	cts: PhantomData<Cts>,
}


impl<S: Sercom> Pads<S> {
	fn default(s: S) -> Self {
		Pads { s, p0: NoneT, p1: NoneT, p2: NoneT, p3: NoneT, rx: PhantomData, tx: PhantomData, clk: PhantomData, rts: PhantomData, cts: PhantomData }
	}
}


pub trait PadsInterface {
    type SercomType: Sercom;
    type P0: OptionalPin;
    type P1: OptionalPin;
    type P2: OptionalPin;
    type P3: OptionalPin;
    type Rx: OptionalPadNum;
    type Tx: OptionalPadNum;
    type Clk: OptionalPadNum;
    type Rts: OptionalPadNum;
    type Cts: OptionalPadNum;

    fn access(self) -> Pads<Self::SercomType, Self::P0, Self::P1, Self::P2, Self::P3, Self::Rx, Self::Tx, Self::Clk, Self::Rts, Self::Cts>;
}


impl<S: Sercom, P0: OptionalPin, P1: OptionalPin, P2: OptionalPin, P3: OptionalPin,
     Rx: OptionalPadNum, Tx: OptionalPadNum, Clk: OptionalPadNum,
     Rts: OptionalPadNum, Cts: OptionalPadNum>
    PadsInterface for Pads<S, P0, P1, P2, P3, Rx, Tx, Clk, Rts, Cts>
{
    type SercomType = S;
    type P0 = P0;
    type P1 = P1;
    type P2 = P2;
    type P3 = P3;
    type Rx = Rx;
    type Tx = Tx;
    type Clk = Clk;
    type Rts = Rts;
    type Cts = Cts;
    fn access(self) -> Pads<S, P0, P1, P2, P3, Rx, Tx, Clk, Rts, Cts> {
        self
    }
}




trait ReplacePad<P: PadNum, NewPin: SomePin> {
	type Output: PadsInterface;
	fn replace_pad(self, new_pin: NewPin) -> Self::Output;
}

impl<S: Sercom, P0: SomePin, P1: OptionalPin, P2: OptionalPin, P3: OptionalPin, Rx: OptionalPadNum, Tx: OptionalPadNum, Clk: OptionalPadNum, Rts: OptionalPadNum, Cts: OptionalPadNum> ReplacePad<Pad0, P0> for Pads<S, NoneT, P1, P2, P3, Rx, Tx, Clk, Rts, Cts> {
	type Output = Pads<S, P0, P1, P2, P3, Rx, Tx, Clk, Rts, Cts>;
	fn replace_pad(self, new_pin: P0) -> Self::Output {
		Pads {
			s: self.s,
			p0: new_pin,
			p1: self.p1,
			p2: self.p2,
			p3: self.p3,
			rx: self.rx,
			tx: self.tx,
			clk: self.clk,
			rts: self.rts,
			cts: self.cts,
		}
	}
}

impl<S: Sercom, P0: OptionalPin, P1: SomePin, P2: OptionalPin, P3: OptionalPin, Rx: OptionalPadNum, Tx: OptionalPadNum, Clk: OptionalPadNum, Rts: OptionalPadNum, Cts: OptionalPadNum> ReplacePad<Pad1, P1> for Pads<S, P0, NoneT, P2, P3, Rx, Tx, Clk, Rts, Cts> {
	type Output = Pads<S, P0, P1, P2, P3, Rx, Tx, Clk, Rts, Cts>;
	fn replace_pad(self, new_pin: P1) -> Self::Output {
		Pads {
			s: self.s,
			p0: self.p0,
			p1: new_pin,
			p2: self.p2,
			p3: self.p3,
			rx: self.rx,
			tx: self.tx,
			clk: self.clk,
			rts: self.rts,
			cts: self.cts,
		}
	}
}

impl<S: Sercom, P0: OptionalPin, P1: OptionalPin, P2: SomePin, P3: OptionalPin, Rx: OptionalPadNum, Tx: OptionalPadNum, Clk: OptionalPadNum, Rts: OptionalPadNum, Cts: OptionalPadNum> ReplacePad<Pad2, P2> for Pads<S, P0, P1, NoneT, P3, Rx, Tx, Clk, Rts, Cts> {
	type Output = Pads<S, P0, P1, P2, P3, Rx, Tx, Clk, Rts, Cts>;
	fn replace_pad(self, new_pin: P2) -> Self::Output {
		Pads {
			s: self.s,
			p0: self.p0,
			p1: self.p1,
			p2: new_pin,
			p3: self.p3,
			rx: self.rx,
			tx: self.tx,
			clk: self.clk,
			rts: self.rts,
			cts: self.cts,
		}
	}
}

impl<S: Sercom, P0: OptionalPin, P1: OptionalPin, P2: OptionalPin, P3: SomePin, Rx: OptionalPadNum, Tx: OptionalPadNum, Clk: OptionalPadNum, Rts: OptionalPadNum, Cts: OptionalPadNum> ReplacePad<Pad3, P3> for Pads<S, P0, P1, P2, NoneT, Rx, Tx, Clk, Rts, Cts> {
	type Output = Pads<S, P0, P1, P2, P3, Rx, Tx, Clk, Rts, Cts>;
	fn replace_pad(self, new_pin: P3) -> Self::Output {
		Pads {
			s: self.s,
			p0: self.p0,
			p1: self.p1,
			p2: self.p2,
			p3: new_pin,
			rx: self.rx,
			tx: self.tx,
			clk: self.clk,
			rts: self.rts,
			cts: self.cts,
		}
	}
}


impl<S: Sercom, P0: OptionalPin, P1: OptionalPin, P2: OptionalPin, P3: OptionalPin, Rx: OptionalPadNum, Tx: OptionalPadNum, Clk: OptionalPadNum, Rts: OptionalPadNum, Cts: OptionalPadNum> Pads<S,P0,P1,P2,P3,Rx,Tx,Clk,Rts,Cts> {
    fn replace_pad<P>(self, new_pin: P) -> <Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output
    where
        P: SomePin,
        P::Id: GetPad<S>,
        Self: ReplacePad<<P::Id as GetPad<S>>::PadNum, P>,
    {
        <Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::replace_pad(self, new_pin)
    }

}

impl<S: Sercom, P0: OptionalPin, P1: OptionalPin, P2: OptionalPin, P3: OptionalPin, Tx: OptionalPadNum, Clk: OptionalPadNum, Rts: OptionalPadNum, Cts: OptionalPadNum> Pads<S,P0,P1,P2,P3,NoneT,Tx,Clk,Rts,Cts> {
	fn rx<P: SomePin>(self, new_pin: P)
	where
		P::Id: pad::GetPad<S>,
		Self: ReplacePad<<P::Id as GetPad<S>>::PadNum, P>,
	{
		let replaced = self.replace_pad(new_pin);
		let a = replaced.access().s;
	}
}


fn test(s: Sercom3, pin: Pin<PA16, AlternateD>, pin2: Pin<PA16, AlternateD>) {
	let tp = Pads::default(s).rx(pin);
}
