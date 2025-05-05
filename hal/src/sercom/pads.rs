use crate::sercom::*;
use crate::typelevel::NoneT;

use core::marker::PhantomData;

//==============================================================================
// Pads
//==============================================================================

pub struct Pads<
    S: Sercom,
    P0: OptionalPad = NoneT,
    P1: OptionalPad = NoneT,
    P2: OptionalPad = NoneT,
    P3: OptionalPad = NoneT,
>(PhantomData<S>, P0, P1, P2, P3);

pub trait IsPads {
    type Sercom: Sercom;
}
impl<S: Sercom, P0: OptionalPad, P1: OptionalPad, P2: OptionalPad, P3: OptionalPad> IsPads
    for Pads<S, P0, P1, P2, P3>
{
    type Sercom = S;
}

impl<S: Sercom> Pads<S, NoneT, NoneT, NoneT, NoneT> {
    pub fn default() -> Self {
        Pads(PhantomData, NoneT, NoneT, NoneT, NoneT)
    }
}

pub trait ReplacePad<I: IsPad>: ReplacePadNum<I, I::PadNum> {}
impl<I: IsPad, R: ReplacePadNum<I, I::PadNum>> ReplacePad<I> for R {}

pub trait ReplacePadNum<I: IsPad, P: PadNum>: IsPads {
    type NewPads: IsPads;
    fn replace(self, pin: I) -> Self::NewPads;
}

impl<
        S: Sercom,
        P1: OptionalPad,
        P2: OptionalPad,
        P3: OptionalPad,
        I: IsPad<PadNum = Pad0, Sercom = S>,
    > ReplacePadNum<I, Pad0> for Pads<S, NoneT, P1, P2, P3>
{
    type NewPads = Pads<S, I, P1, P2, P3>;
    fn replace(self, pin: I) -> Self::NewPads {
        Pads(self.0, pin, self.2, self.3, self.4)
    }
}

impl<
        S: Sercom,
        P0: OptionalPad,
        P2: OptionalPad,
        P3: OptionalPad,
        I: IsPad<PadNum = Pad1, Sercom = S>,
    > ReplacePadNum<I, Pad1> for Pads<S, P0, NoneT, P2, P3>
{
    type NewPads = Pads<S, P0, I, P2, P3>;
    fn replace(self, pin: I) -> Self::NewPads {
        Pads(self.0, self.1, pin, self.3, self.4)
    }
}

impl<
        S: Sercom,
        P0: OptionalPad,
        P1: OptionalPad,
        P3: OptionalPad,
        I: IsPad<PadNum = Pad2, Sercom = S>,
    > ReplacePadNum<I, Pad2> for Pads<S, P0, P1, NoneT, P3>
{
    type NewPads = Pads<S, P0, P1, I, P3>;
    fn replace(self, pin: I) -> Self::NewPads {
        Pads(self.0, self.1, self.2, pin, self.4)
    }
}

impl<
        S: Sercom,
        P0: OptionalPad,
        P1: OptionalPad,
        P2: OptionalPad,
        I: IsPad<PadNum = Pad3, Sercom = S>,
    > ReplacePadNum<I, Pad3> for Pads<S, P0, P1, P2, NoneT>
{
    type NewPads = Pads<S, P0, P1, P2, I>;
    fn replace(self, pin: I) -> Self::NewPads {
        Pads(self.0, self.1, self.2, self.3, pin)
    }
}

pub trait ValidPads: IsPads {}

#[hal_cfg(any("sercom0-d11", "sercom0-d21"))]
impl<S: Sercom, P0: OptionalPad, P1: OptionalPad, P2: OptionalPad, P3: OptionalPad> ValidPads
    for Pads<S, P0, P1, P2, P3>
{
}

#[hal_cfg("sercom0-d5x")]
impl<S: Sercom, P0: OptionalPad, P1: OptionalPad, P2: OptionalPad, P3: OptionalPad> ValidPads
    for Pads<S, P0, P1, P2, P3>
where
    (P0, P1, P2, P3): ShareIoSet,
{
}
