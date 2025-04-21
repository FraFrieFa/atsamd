use crate::typelevel::NoneT;
use crate::sercom::*;

//==============================================================================
// Pads
//==============================================================================

pub struct Pads<P0: OptionalPad, P1: OptionalPad, P2: OptionalPad, P3: OptionalPad> (P0, P1, P2, P3);

pub trait IsPads {}
impl<P0: OptionalPad, P1: OptionalPad, P2: OptionalPad, P3: OptionalPad> IsPads
    for Pads<P0, P1, P2, P3>
{
}

impl Pads<NoneT, NoneT, NoneT, NoneT> {
	pub fn default() -> Self {
		Pads(NoneT, NoneT, NoneT, NoneT)
	}
}

pub trait ReplacePad<P: PadNum> : IsPads
{
	type NewPads<I: IsPad>: IsPads;
    fn replace<I: IsPad<PadNum = P>>(self, pin: I) -> Self::NewPads<I>;
}

impl<P1: OptionalPad, P2: OptionalPad, P3: OptionalPad> ReplacePad<Pad0>
    for Pads<NoneT, P1, P2, P3>
{
	type NewPads<I: IsPad> = Pads<I, P1, P2, P3>;
    fn replace<I: IsPad<PadNum = Pad0>>(self, pin: I) -> Self::NewPads<I> {
        Pads(pin, self.1, self.2, self.3)
    }
}

impl<P0: OptionalPad, P2: OptionalPad, P3: OptionalPad> ReplacePad<Pad1>
    for Pads<P0, NoneT, P2, P3>
{
	type NewPads<I: IsPad> = Pads<P0, I, P2, P3>;
    fn replace<I: IsPad<PadNum = Pad1>>(self, pin: I) -> Self::NewPads<I> {
        Pads(self.0, pin, self.2, self.3)
    }
}

impl<P0: OptionalPad, P1: OptionalPad, P3: OptionalPad> ReplacePad<Pad2>
    for Pads<P0, P1, NoneT, P3>
{
	type NewPads<I: IsPad> = Pads<P0, P1, I, P3>;
    fn replace<I: IsPad<PadNum = Pad2>>(self, pin: I) -> Self::NewPads<I> {
        Pads(self.0, self.1, pin, self.3)
    }
}

impl<P0: OptionalPad, P1: OptionalPad, P2: OptionalPad> ReplacePad<Pad3>
    for Pads<P0, P1, P2, NoneT>
{
	type NewPads<I: IsPad> = Pads<P0, P1, P2, I>;
    fn replace<I: IsPad<PadNum = Pad3>>(self, pin: I) -> Self::NewPads<I> {
        Pads(self.0, self.1, self.2, pin)
    }
}
