use crate::sercom;
use crate::sercom::pads;
use crate::sercom::pads::{ReplacePad, ReplacePadNum};
use crate::sercom::uart::{AnyConfig, Capability, CharSize, Config};
use crate::sercom::{uart, IsPad, OptionalPad, Sercom};
use crate::typelevel::NoneT;

use atsamd_hal_macros::hal_cfg;
use core::marker::PhantomData;

type DefaultPads<S> = Pads<pads::Pads<S>, Roles>;
type AddRole<P, R, NP> = Pads<
    <<P as IsPads>::Pads as ReplacePadNum<NP, <NP as IsPad>::PadNum>>::NewPads,
    <<P as IsPads>::Roles as ReplaceRole<R>>::NewRoles<NP>,
>;

pub type PadsFromRxTx<S, RX, TX> = AddRole<AddRole<DefaultPads<S>, Rx, RX>, Tx, TX>;
pub type PadsFromRxTxRtsCts<S, RX, TX, RTS, CTS> =
    AddRole<AddRole<AddRole<AddRole<DefaultPads<S>, Rx, RX>, Tx, TX>, Rts, RTS>, Cts, CTS>;

#[hal_cfg(any("sercom0-d21", "sercom0-d5x"))]
pub type PadsFromPinsRxTx<S, RX, TX> =
    PadsFromRxTx<S, <RX as sercom::GetOptionalPad<S>>::Pad, <TX as sercom::GetOptionalPad<S>>::Pad>;

pub struct Roles<
    RX: OptionalPad = NoneT,
    TX: OptionalPad = NoneT,
    CLK: OptionalPad = NoneT,
    RTS: OptionalPad = NoneT,
    CTS: OptionalPad = NoneT,
>(
    PhantomData<RX>,
    PhantomData<TX>,
    PhantomData<CLK>,
    PhantomData<RTS>,
    PhantomData<CTS>,
);
pub trait IsRoles {}
impl<RX: OptionalPad, TX: OptionalPad, CLK: OptionalPad, RTS: OptionalPad, CTS: OptionalPad> IsRoles
    for Roles<RX, TX, CLK, RTS, CTS>
{
}

impl Default for Roles {
    fn default() -> Self {
        Roles(
            PhantomData,
            PhantomData,
            PhantomData,
            PhantomData,
            PhantomData,
        )
    }
}

pub struct Rx;
pub struct Tx;
pub struct Clk;
pub struct Rts;
pub struct Cts;

pub trait IsRole {}
impl IsRole for Rx {}
impl IsRole for Tx {}
impl IsRole for Clk {}
impl IsRole for Rts {}
impl IsRole for Cts {}

pub trait ReplaceRole<R: IsRole>: IsRoles {
    type NewRoles<I: IsPad>: IsRoles;
    fn replace<I: IsPad>(self) -> Self::NewRoles<I>;
}

impl<TX: OptionalPad, CLK: OptionalPad, RTS: OptionalPad, CTS: OptionalPad> ReplaceRole<Rx>
    for Roles<NoneT, TX, CLK, RTS, CTS>
{
    type NewRoles<I: IsPad> = Roles<I, TX, CLK, RTS, CTS>;
    fn replace<I: IsPad>(self) -> Self::NewRoles<I> {
        Roles(PhantomData::<I>, self.1, self.2, self.3, self.4)
    }
}

impl<RX: OptionalPad, CLK: OptionalPad, RTS: OptionalPad, CTS: OptionalPad> ReplaceRole<Tx>
    for Roles<RX, NoneT, CLK, RTS, CTS>
{
    type NewRoles<I: IsPad> = Roles<RX, I, CLK, RTS, CTS>;
    fn replace<I: IsPad>(self) -> Self::NewRoles<I> {
        Roles(self.0, PhantomData::<I>, self.2, self.3, self.4)
    }
}

impl<RX: OptionalPad, TX: OptionalPad, RTS: OptionalPad, CTS: OptionalPad> ReplaceRole<Clk>
    for Roles<RX, TX, NoneT, RTS, CTS>
{
    type NewRoles<I: IsPad> = Roles<RX, TX, I, RTS, CTS>;
    fn replace<I: IsPad>(self) -> Self::NewRoles<I> {
        Roles(self.0, self.1, PhantomData::<I>, self.3, self.4)
    }
}

impl<RX: OptionalPad, TX: OptionalPad, CLK: OptionalPad, CTS: OptionalPad> ReplaceRole<Rts>
    for Roles<RX, TX, CLK, NoneT, CTS>
{
    type NewRoles<I: IsPad> = Roles<RX, TX, CLK, I, CTS>;
    fn replace<I: IsPad>(self) -> Self::NewRoles<I> {
        Roles(self.0, self.1, self.2, PhantomData::<I>, self.4)
    }
}

impl<RX: OptionalPad, TX: OptionalPad, CLK: OptionalPad, RTS: OptionalPad> ReplaceRole<Cts>
    for Roles<RX, TX, CLK, RTS, NoneT>
{
    type NewRoles<I: IsPad> = Roles<RX, TX, CLK, RTS, I>;
    fn replace<I: IsPad>(self) -> Self::NewRoles<I> {
        Roles(self.0, self.1, self.2, self.3, PhantomData::<I>)
    }
}

pub struct Pads<P: pads::ValidPads, R: IsRoles = Roles<NoneT, NoneT, NoneT, NoneT, NoneT>> {
    pads: P,
    roles: R,
}

pub trait IsPads {
    type Pads: pads::ValidPads;
    type Roles: IsRoles;
}
impl<P: pads::ValidPads, R: IsRoles> IsPads for Pads<P, R> {
    type Pads = P;
    type Roles = R;
}

impl<S: Sercom> Default for Pads<pads::Pads<S, NoneT, NoneT, NoneT, NoneT>> {
    fn default() -> Self {
        Pads {
            pads: pads::Pads::<S>::default(),
            roles: Roles::default(),
        }
    }
}

impl<P: pads::ValidPads, R: IsRoles> Pads<P, R> {
    pub fn rx<I: IsPad>(self, pin: I) -> Pads<P::NewPads, R::NewRoles<I>>
    where
        P: ReplacePad<I>,
        R: ReplaceRole<Rx>,
        P::NewPads: pads::ValidPads,
    {
        Pads {
            pads: self.pads.replace(pin),
            roles: self.roles.replace::<I>(),
        }
    }
    pub fn tx<I: IsPad>(self, pin: I) -> Pads<P::NewPads, R::NewRoles<I>>
    where
        P: ReplacePad<I>,
        R: ReplaceRole<Tx>,
        P::NewPads: pads::ValidPads,
    {
        Pads {
            pads: self.pads.replace(pin),
            roles: self.roles.replace::<I>(),
        }
    }
    pub fn io<I: IsPad>(
        self,
        pin: I,
    ) -> Pads<P::NewPads, <R::NewRoles<I> as ReplaceRole<Tx>>::NewRoles<I>>
    where
        P: ReplacePad<I>,
        R: ReplaceRole<Rx>,
        R::NewRoles<I>: ReplaceRole<Tx>,
        P::NewPads: pads::ValidPads,
    {
        Pads {
            pads: self.pads.replace(pin),
            roles: self.roles.replace::<I>().replace::<I>(),
        }
    }
    pub fn clk<I: IsPad>(self, pin: I) -> Pads<P::NewPads, R::NewRoles<I>>
    where
        P: ReplacePad<I>,
        R: ReplaceRole<Clk>,
        P::NewPads: pads::ValidPads,
    {
        Pads {
            pads: self.pads.replace(pin),
            roles: self.roles.replace::<I>(),
        }
    }
    pub fn rts<I: IsPad>(self, pin: I) -> Pads<P::NewPads, R::NewRoles<I>>
    where
        P: ReplacePad<I>,
        R: ReplaceRole<Rts>,
        P::NewPads: pads::ValidPads,
    {
        Pads {
            pads: self.pads.replace(pin),
            roles: self.roles.replace::<I>(),
        }
    }
    pub fn cts<I: IsPad>(self, pin: I) -> Pads<P::NewPads, R::NewRoles<I>>
    where
        P: ReplacePad<I>,
        R: ReplaceRole<Cts>,
        P::NewPads: pads::ValidPads,
    {
        Pads {
            pads: self.pads.replace(pin),
            roles: self.roles.replace::<I>(),
        }
    }
}

use crate::sercom::{Pad0, Pad1, Pad2, Pad3};

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

trait Rxpo {
    const RXPO: u8;
}

impl_const! {
    trait = Rxpo;
    field = const RXPO: u8;
    NoneT => 0,
    Pad0 => 0,
    Pad1 => 1,
    Pad2 => 2,
    Pad3 => 3,
}

trait Txpo {
    const TXPO: u8;
}

#[hal_cfg(any("sercom0-d11", "sercom0-d21"))]
impl_const! {
    trait = Txpo;
    field = const TXPO: u8;
    (NoneT, NoneT, NoneT, NoneT) => 0,
    (NoneT, Pad1, NoneT, NoneT) => 0,
    (Pad0, NoneT, NoneT, NoneT) => 0,
    (Pad0, Pad1, NoneT, NoneT) => 0,
    (NoneT, Pad3, NoneT, NoneT) => 1,
    (Pad2, NoneT, NoneT, NoneT) => 1,
    (Pad2, Pad3, NoneT, NoneT) => 1,
    (NoneT, NoneT, Pad2, Pad3) => 2,
    (Pad0, NoneT, NoneT, Pad3) => 2,
    (Pad0, NoneT, Pad2, NoneT) => 2,
    (NoneT, NoneT, NoneT, Pad3) => 2,
    (NoneT, NoneT, Pad2, NoneT) => 2,
    (Pad0, NoneT, Pad2, Pad3) => 2,
}

#[hal_cfg("sercom0-d5x")]
impl_const! {
    trait = Txpo;
    field = const TXPO: u8;
    (NoneT, NoneT, NoneT, NoneT) => 0,
    (NoneT, Pad1, NoneT, NoneT) => 0,
    (Pad0, NoneT, NoneT, NoneT) => 0,
    (Pad0, Pad1, NoneT, NoneT) => 0,
    (NoneT, NoneT, NoneT, Pad3) => 2,
    (Pad0, NoneT, NoneT, Pad3) => 2,
    (NoneT, NoneT, Pad2, Pad3) => 2,
    (Pad0, NoneT, Pad2, Pad3) => 2,
    (Pad0, NoneT, Pad2, NoneT) => 3,
    (NoneT, NoneT, Pad2, NoneT) => 3,
    (NoneT, Pad1, Pad2, NoneT) => 3,
    (Pad0, Pad1, Pad2, NoneT) => 3,
}

pub trait CapabilityRxTx {
    type Capability: Capability;
}

impl<RX: IsPad> CapabilityRxTx for (RX, NoneT) {
    type Capability = uart::Rx;
}
impl<TX: IsPad> CapabilityRxTx for (NoneT, TX) {
    type Capability = uart::Tx;
}
impl<RX: IsPad, TX: IsPad> CapabilityRxTx for (RX, TX) {
    type Capability = uart::Duplex;
}

pub trait ValidPads {
    const RXPO: u8;
    const TXPO: u8;
    type Capability: Capability;
    type Sercom: Sercom;
    type CTS: OptionalPad;
}

impl<
        P: pads::ValidPads,
        RX: OptionalPad,
        TX: OptionalPad,
        CLK: OptionalPad,
        RTS: OptionalPad,
        CTS: OptionalPad,
    > ValidPads for Pads<P, Roles<RX, TX, CLK, RTS, CTS>>
where
    RX::PadNum: Rxpo,
    (TX::PadNum, CLK::PadNum, RTS::PadNum, CTS::PadNum): Txpo,
    (RX, TX): CapabilityRxTx,
{
    const RXPO: u8 = RX::PadNum::RXPO;
    const TXPO: u8 = <(TX::PadNum, CLK::PadNum, RTS::PadNum, CTS::PadNum)>::TXPO;
    type Capability = <(RX, TX) as CapabilityRxTx>::Capability;
    type Sercom = P::Sercom;
    type CTS = CTS;
}

//=============================================================================
// ValidConfig
//=============================================================================

/// Marker trait for valid UART [`Config`]urations
///
/// A functional UART peripheral must have, at a minimum either a Rx or a Tx
/// [`Pad`].
pub trait ValidConfig: AnyConfig {}

impl<P: ValidPads, C: CharSize> ValidConfig for Config<P, C> {}
