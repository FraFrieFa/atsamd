//! Shared owning pad abstractions for the new SERCOM package.

use crate::sercom_v2::{IsPad, OptionalPad, Pad0, Pad1, Pad2, Pad3, PadNum, Sercom};
use crate::typelevel::NoneT;

pub struct Pads<
    P0: OptionalPad = NoneT,
    P1: OptionalPad = NoneT,
    P2: OptionalPad = NoneT,
    P3: OptionalPad = NoneT,
>(pub P0, pub P1, pub P2, pub P3);

impl Default for Pads<NoneT, NoneT, NoneT, NoneT> {
    fn default() -> Self {
        Self(NoneT, NoneT, NoneT, NoneT)
    }
}

pub trait ValidPads {
    type Sercom: Sercom;
}

impl<P0: IsPad, P1: OptionalPad, P2: OptionalPad, P3: OptionalPad> ValidPads
    for Pads<P0, P1, P2, P3>
{
    type Sercom = P0::Sercom;
}

impl<P1: IsPad, P2: OptionalPad, P3: OptionalPad> ValidPads for Pads<NoneT, P1, P2, P3> {
    type Sercom = P1::Sercom;
}

impl<P2: IsPad, P3: OptionalPad> ValidPads for Pads<NoneT, NoneT, P2, P3> {
    type Sercom = P2::Sercom;
}

impl<P3: IsPad> ValidPads for Pads<NoneT, NoneT, NoneT, P3> {
    type Sercom = P3::Sercom;
}

pub trait ReplacePad<I: OptionalPad> {
    type NewPads;
    fn replace(self, pin: I) -> Self::NewPads;
}

pub trait ReplacePadNum<I: IsPad, N: PadNum> {
    type NewPads;
    fn replace(self, pin: I) -> Self::NewPads;
}

impl<I: IsPad, R: ReplacePadNum<I, I::PadNum>> ReplacePad<I> for R {
    type NewPads = R::NewPads;

    fn replace(self, pin: I) -> Self::NewPads {
        <R as ReplacePadNum<I, I::PadNum>>::replace(self, pin)
    }
}

impl<V: ValidPads> ReplacePad<NoneT> for V {
    type NewPads = V;

    fn replace(self, _: NoneT) -> Self::NewPads {
        self
    }
}

impl<P1: OptionalPad, P2: OptionalPad, P3: OptionalPad, I: IsPad<PadNum = Pad0>>
    ReplacePadNum<I, Pad0> for Pads<NoneT, P1, P2, P3>
{
    type NewPads = Pads<I, P1, P2, P3>;

    fn replace(self, pin: I) -> Self::NewPads {
        Pads(pin, self.1, self.2, self.3)
    }
}

impl<P0: OptionalPad, P2: OptionalPad, P3: OptionalPad, I: IsPad<PadNum = Pad1>>
    ReplacePadNum<I, Pad1> for Pads<P0, NoneT, P2, P3>
{
    type NewPads = Pads<P0, I, P2, P3>;

    fn replace(self, pin: I) -> Self::NewPads {
        Pads(self.0, pin, self.2, self.3)
    }
}

impl<P0: OptionalPad, P1: OptionalPad, P3: OptionalPad, I: IsPad<PadNum = Pad2>>
    ReplacePadNum<I, Pad2> for Pads<P0, P1, NoneT, P3>
{
    type NewPads = Pads<P0, P1, I, P3>;

    fn replace(self, pin: I) -> Self::NewPads {
        Pads(self.0, self.1, pin, self.3)
    }
}

impl<P0: OptionalPad, P1: OptionalPad, P2: OptionalPad, I: IsPad<PadNum = Pad3>>
    ReplacePadNum<I, Pad3> for Pads<P0, P1, P2, NoneT>
{
    type NewPads = Pads<P0, P1, P2, I>;

    fn replace(self, pin: I) -> Self::NewPads {
        Pads(self.0, self.1, self.2, pin)
    }
}

macro_rules! impl_const {
    (
        trait = $trait:path;
        field = const $field:ident : $ty:ty;
        $(
            $impl_ty:ty => $value:expr
        ),* $(,)?
    ) => {
        $(
            impl $trait for $impl_ty {
                const $field: $ty = $value;
            }
        )*
    };
}

pub(crate) use impl_const;

pub trait IsPadSet {
    type Pads;
    type Roles;
}

pub trait CapabilityMarker {}
