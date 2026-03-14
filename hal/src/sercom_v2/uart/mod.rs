//! USART baseline for the new SERCOM package.

use core::marker::PhantomData;

use atsamd_hal_macros::hal_cfg;
#[cfg(any(
    feature = "samd21e",
    feature = "samd21g",
    feature = "samd21j",
    feature = "samd21el",
    feature = "samd21gl",
    feature = "samd51g",
    feature = "samd51j",
    feature = "samd51n",
    feature = "samd51p",
    feature = "same51g",
    feature = "same51j",
    feature = "same51n",
    feature = "same53j",
    feature = "same53n",
    feature = "same54n",
    feature = "same54p"
))]
use sorted_hlist::{HCons, HList, IntersectUnchecked, NonEmptyHList, mk_hlist};

use crate::gpio::{Pin, PinId, PinMode};
use crate::sercom_v2::{IsPad, OptionalPad, Pad0, Pad1, Pad2, Pad3, Sercom, SercomCoreClock};
#[cfg(any(
    feature = "samd21e",
    feature = "samd21g",
    feature = "samd21j",
    feature = "samd21el",
    feature = "samd21gl",
    feature = "samd51g",
    feature = "samd51j",
    feature = "samd51n",
    feature = "samd51p",
    feature = "same51g",
    feature = "same51j",
    feature = "same51n",
    feature = "same53j",
    feature = "same53n",
    feature = "same54n",
    feature = "same54p"
))]
use crate::sercom_v2::{PinSercoms, SercomOrder};
use crate::typelevel::NoneT;

use super::pads;
use super::pads::{IsPadSet, ReplacePad};
use super::{
    HasApbClkCtrl, PacApbAccess, PacTakeSercom, ResourceSet, StateMarker, TakeSercom,
    TakeSercomCoreClock,
};

#[hal_cfg(any("sercom0-d11", "sercom0-d21"))]
use crate::pac::sercom0::usart::ctrla::Modeselect;

#[hal_cfg("sercom0-d5x")]
use crate::pac::sercom0::usart_int::ctrla::Modeselect;

/// Semantic USART state markers.
pub trait UsartState: StateMarker {}

pub enum AnyMode {}
pub enum Async {}
pub enum Sync {}
pub enum LinHost {}
pub enum Iso7816 {}
pub enum FullDuplex {}
pub enum HalfDuplex {}

impl StateMarker for AnyMode {}
impl StateMarker for Async {}
impl StateMarker for Sync {}
impl StateMarker for LinHost {}
impl StateMarker for Iso7816 {}
impl StateMarker for FullDuplex {}
impl StateMarker for HalfDuplex {}

impl UsartState for AnyMode {}
impl UsartState for Async {}
impl UsartState for Sync {}
impl UsartState for LinHost {}
impl UsartState for Iso7816 {}
impl UsartState for FullDuplex {}
impl UsartState for HalfDuplex {}

pub enum Rx {}
pub enum Tx {}
pub enum Duplex {}

impl super::pads::CapabilityMarker for Rx {}
impl super::pads::CapabilityMarker for Tx {}
impl super::pads::CapabilityMarker for Duplex {}

type DefaultPads = InternalPads<pads::Pads, Roles>;
type AddRole<P, R, NP> = InternalPads<
    <<P as IsPadSet>::Pads as ReplacePad<NP>>::NewPads,
    <<P as IsPadSet>::Roles as ReplaceRole<R>>::NewRoles<NP>,
>;

pub type Pads<RX = NoneT, TX = NoneT, RTS = NoneT, CTS = NoneT> = AddRole<
    AddRole<AddRole<AddRole<DefaultPads, RxRole, RX>, TxRole, TX>, RtsRole, RTS>,
    CtsRole,
    CTS,
>;

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

impl<RX: OptionalPad, TX: OptionalPad, CLK: OptionalPad, RTS: OptionalPad, CTS: OptionalPad>
    IsRoles for Roles<RX, TX, CLK, RTS, CTS>
{
}

impl<RX: OptionalPad, TX: OptionalPad, CLK: OptionalPad, RTS: OptionalPad, CTS: OptionalPad>
    Default for Roles<RX, TX, CLK, RTS, CTS>
{
    fn default() -> Self {
        Self(
            PhantomData,
            PhantomData,
            PhantomData,
            PhantomData,
            PhantomData,
        )
    }
}

pub struct RxRole;
pub struct TxRole;
pub struct ClkRole;
pub struct RtsRole;
pub struct CtsRole;

pub trait ReplaceRole<R> {
    type NewRoles<I: OptionalPad>;
    fn replace<I: OptionalPad>(self) -> Self::NewRoles<I>;
}

impl<TX: OptionalPad, CLK: OptionalPad, RTS: OptionalPad, CTS: OptionalPad> ReplaceRole<RxRole>
    for Roles<NoneT, TX, CLK, RTS, CTS>
{
    type NewRoles<I: OptionalPad> = Roles<I, TX, CLK, RTS, CTS>;
    fn replace<I: OptionalPad>(self) -> Self::NewRoles<I> {
        Roles(
            PhantomData,
            PhantomData,
            PhantomData,
            PhantomData,
            PhantomData,
        )
    }
}
impl<RX: OptionalPad, CLK: OptionalPad, RTS: OptionalPad, CTS: OptionalPad> ReplaceRole<TxRole>
    for Roles<RX, NoneT, CLK, RTS, CTS>
{
    type NewRoles<I: OptionalPad> = Roles<RX, I, CLK, RTS, CTS>;
    fn replace<I: OptionalPad>(self) -> Self::NewRoles<I> {
        Roles(
            PhantomData,
            PhantomData,
            PhantomData,
            PhantomData,
            PhantomData,
        )
    }
}
impl<RX: OptionalPad, TX: OptionalPad, RTS: OptionalPad, CTS: OptionalPad> ReplaceRole<ClkRole>
    for Roles<RX, TX, NoneT, RTS, CTS>
{
    type NewRoles<I: OptionalPad> = Roles<RX, TX, I, RTS, CTS>;
    fn replace<I: OptionalPad>(self) -> Self::NewRoles<I> {
        Roles(
            PhantomData,
            PhantomData,
            PhantomData,
            PhantomData,
            PhantomData,
        )
    }
}
impl<RX: OptionalPad, TX: OptionalPad, CLK: OptionalPad, CTS: OptionalPad> ReplaceRole<RtsRole>
    for Roles<RX, TX, CLK, NoneT, CTS>
{
    type NewRoles<I: OptionalPad> = Roles<RX, TX, CLK, I, CTS>;
    fn replace<I: OptionalPad>(self) -> Self::NewRoles<I> {
        Roles(
            PhantomData,
            PhantomData,
            PhantomData,
            PhantomData,
            PhantomData,
        )
    }
}
impl<RX: OptionalPad, TX: OptionalPad, CLK: OptionalPad, RTS: OptionalPad> ReplaceRole<CtsRole>
    for Roles<RX, TX, CLK, RTS, NoneT>
{
    type NewRoles<I: OptionalPad> = Roles<RX, TX, CLK, RTS, I>;
    fn replace<I: OptionalPad>(self) -> Self::NewRoles<I> {
        Roles(
            PhantomData,
            PhantomData,
            PhantomData,
            PhantomData,
            PhantomData,
        )
    }
}

pub struct InternalPads<
    P = pads::Pads<NoneT, NoneT, NoneT, NoneT>,
    R: IsRoles = Roles<NoneT, NoneT, NoneT, NoneT, NoneT>,
> {
    pads: P,
    roles: R,
}

impl<P, R: IsRoles> IsPadSet for InternalPads<P, R> {
    type Pads = P;
    type Roles = R;
}

impl Default for InternalPads<pads::Pads<NoneT, NoneT, NoneT, NoneT>> {
    fn default() -> Self {
        Self {
            pads: pads::Pads::default(),
            roles: Roles::default(),
        }
    }
}

impl<P, R: IsRoles> InternalPads<P, R> {
    pub fn rx<I: IsPad>(self, pin: I) -> InternalPads<P::NewPads, R::NewRoles<I>>
    where
        P: ReplacePad<I>,
        P::NewPads: pads::ValidPads,
        R: ReplaceRole<RxRole>,
        R::NewRoles<I>: IsRoles,
    {
        InternalPads {
            pads: self.pads.replace(pin),
            roles: self.roles.replace::<I>(),
        }
    }

    pub fn tx<I: IsPad>(self, pin: I) -> InternalPads<P::NewPads, R::NewRoles<I>>
    where
        P: ReplacePad<I>,
        P::NewPads: pads::ValidPads,
        R: ReplaceRole<TxRole>,
        R::NewRoles<I>: IsRoles,
    {
        InternalPads {
            pads: self.pads.replace(pin),
            roles: self.roles.replace::<I>(),
        }
    }

    pub fn io<I: IsPad>(
        self,
        pin: I,
    ) -> InternalPads<P::NewPads, <R::NewRoles<I> as ReplaceRole<TxRole>>::NewRoles<I>>
    where
        P: ReplacePad<I>,
        P::NewPads: pads::ValidPads,
        R: ReplaceRole<RxRole>,
        R::NewRoles<I>: ReplaceRole<TxRole>,
        <R::NewRoles<I> as ReplaceRole<TxRole>>::NewRoles<I>: IsRoles,
    {
        InternalPads {
            pads: self.pads.replace(pin),
            roles: self.roles.replace::<I>().replace::<I>(),
        }
    }

    pub fn clk<I: IsPad>(self, pin: I) -> InternalPads<P::NewPads, R::NewRoles<I>>
    where
        P: ReplacePad<I>,
        P::NewPads: pads::ValidPads,
        R: ReplaceRole<ClkRole>,
        R::NewRoles<I>: IsRoles,
    {
        InternalPads {
            pads: self.pads.replace(pin),
            roles: self.roles.replace::<I>(),
        }
    }

    pub fn rts<I: IsPad>(self, pin: I) -> InternalPads<P::NewPads, R::NewRoles<I>>
    where
        P: ReplacePad<I>,
        P::NewPads: pads::ValidPads,
        R: ReplaceRole<RtsRole>,
        R::NewRoles<I>: IsRoles,
    {
        InternalPads {
            pads: self.pads.replace(pin),
            roles: self.roles.replace::<I>(),
        }
    }

    pub fn cts<I: IsPad>(self, pin: I) -> InternalPads<P::NewPads, R::NewRoles<I>>
    where
        P: ReplacePad<I>,
        P::NewPads: pads::ValidPads,
        R: ReplaceRole<CtsRole>,
        R::NewRoles<I>: IsRoles,
    {
        InternalPads {
            pads: self.pads.replace(pin),
            roles: self.roles.replace::<I>(),
        }
    }
}

trait Rxpo {
    const RXPO: u8;
}

super::pads::impl_const! {
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

#[cfg(any(feature = "samd11c", feature = "samd11d", feature = "samd21e", feature = "samd21g", feature = "samd21j", feature = "samd21gl", feature = "samd21el"))]
super::pads::impl_const! {
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

#[cfg(any(feature = "samd51g", feature = "samd51j", feature = "samd51n", feature = "samd51p", feature = "same51g", feature = "same51j", feature = "same51n", feature = "same53j", feature = "same53n", feature = "same54n", feature = "same54p"))]
super::pads::impl_const! {
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

pub trait ValidPads {
    const RXPO: u8;
    const TXPO: u8;
    type Capability: super::pads::CapabilityMarker;
    type Sercom: Sercom;
    type CTS: OptionalPad;
}

pub trait CapabilityOf {
    type Capability: super::pads::CapabilityMarker;
}

impl<RX: IsPad> CapabilityOf for (RX, NoneT) {
    type Capability = Rx;
}

impl<TX: IsPad> CapabilityOf for (NoneT, TX) {
    type Capability = Tx;
}

impl<RX: IsPad, TX: IsPad> CapabilityOf for (RX, TX) {
    type Capability = Duplex;
}

impl<P: pads::ValidPads, RX: OptionalPad, TX: OptionalPad, CLK: OptionalPad, RTS: OptionalPad, CTS: OptionalPad>
    ValidPads for InternalPads<P, Roles<RX, TX, CLK, RTS, CTS>>
where
    <RX as OptionalPad>::PadNum: Rxpo,
    (<TX as OptionalPad>::PadNum, <CLK as OptionalPad>::PadNum, <RTS as OptionalPad>::PadNum, <CTS as OptionalPad>::PadNum): Txpo,
    (RX, TX): CapabilityOf,
{
    const RXPO: u8 = <RX as OptionalPad>::PadNum::RXPO;
    const TXPO: u8 =
        <(<TX as OptionalPad>::PadNum, <CLK as OptionalPad>::PadNum, <RTS as OptionalPad>::PadNum, <CTS as OptionalPad>::PadNum)>::TXPO;
    type Capability = <(RX, TX) as CapabilityOf>::Capability;
    type Sercom = P::Sercom;
    type CTS = CTS;
}

/// Owned resources that justify USART pad and clock state.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UsartResources<Pads = InternalPads, Clock = (), Dma = (), Irqs = ()> {
    pub pads: Pads,
    pub clock: Clock,
    pub dma: Dma,
    pub irqs: Irqs,
}

impl<Pads, Clock, Dma, Irqs> ResourceSet for UsartResources<Pads, Clock, Dma, Irqs> {}

/// Phantom routing state justified by owned pads.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PadRouting<Rxpo = (), Txpo = (), Clk = (), Cts = ()>(
    PhantomData<(Rxpo, Txpo, Clk, Cts)>,
);

/// Minimal enabled USART peripheral for the first end-to-end vertical slice.
pub struct BasicUsart<S: Sercom, Capability, Pads = InternalPads, Clock = (), Dma = (), Irqs = ()>
{
    sercom: S,
    resources: UsartResources<Pads, Clock, Dma, Irqs>,
    runtime: UsartRuntime,
    _capability: PhantomData<Capability>,
}

pub trait CapabilityFlags {
    const RX: bool;
    const TX: bool;
}

impl CapabilityFlags for Rx {
    const RX: bool = true;
    const TX: bool = false;
}

impl CapabilityFlags for Tx {
    const RX: bool = false;
    const TX: bool = true;
}

impl CapabilityFlags for Duplex {
    const RX: bool = true;
    const TX: bool = true;
}

#[hal_cfg(any("sercom0-d11", "sercom0-d21"))]
#[inline]
fn usart<S: Sercom>(sercom: &S) -> &crate::pac::sercom0::Usart {
    sercom.usart()
}

#[hal_cfg("sercom0-d5x")]
#[inline]
fn usart<S: Sercom>(sercom: &S) -> &crate::pac::sercom0::UsartInt {
    sercom.usart_int()
}

#[inline]
fn calculate_baud_asynchronous_arithm(baudrate: u32, clk_freq: u32, n_samples: u8) -> u16 {
    const SHIFT: u8 = 32;
    let sample_rate = (n_samples as u64 * baudrate as u64) << SHIFT;
    let ratio = sample_rate / clk_freq as u64;
    let scale = (1u64 << SHIFT) - ratio;
    let baud_calculated = (65536u64 * scale) >> SHIFT;
    baud_calculated as u16
}

impl<S, Pads, Clock, Dma, Irqs>
    UsartConfig<S, super::Disabled, UsartResources<Pads, Clock, Dma, Irqs>, UsartRuntime>
where
    S: Sercom,
    Pads: ValidPads<Sercom = S>,
    Pads::Capability: CapabilityFlags,
    Clock: SercomCoreClock<S>,
{
    /// Configure and enable a basic asynchronous USART mode (8N1).
    ///
    /// This is the first end-to-end implementation path in the new SERCOM API.
    /// It configures mode, RXPO/TXPO and BAUD, then enables RX/TX according to
    /// the pad capability.
    pub fn enable_basic(
        self,
        mut sercom: S,
        apb: &super::ApbClkCtrl,
    ) -> BasicUsart<S, Pads::Capability, Pads, Clock, Dma, Irqs> {
        let core_clock_hz = self.resources.clock.freq_hz();
        sercom.enable_apb_clock(apb);
        let usart = usart(&sercom);

        // Reset and wait for synchronization.
        usart.ctrla().write(|w| w.swrst().set_bit());
        while usart.syncbusy().read().swrst().bit_is_set() {}

        // Internal clock USART mode and fixed routing derived from typed pads.
        usart
            .ctrla()
            .modify(|_, w| w.mode().variant(Modeselect::UsartIntClk));
        usart.ctrla().modify(|_, w| unsafe {
            w.rxpo().bits(Pads::RXPO);
            w.txpo().bits(Pads::TXPO)
        });

        // Start with a strict baseline: async, no parity, 8 data bits.
        usart.ctrla().modify(|_, w| unsafe { w.form().bits(0) });
        usart.ctrla().modify(|_, w| w.cmode().clear_bit());
        usart.ctrlb().modify(|_, w| unsafe { w.chsize().bits(0) });

        if let Some(dord) = self.runtime.dord {
            usart.ctrla().modify(|_, w| w.dord().bit(dord));
        }
        if let Some(runstdby) = self.runtime.runstdby {
            usart.ctrla().modify(|_, w| w.runstdby().bit(runstdby));
        }
        if let Some(ibon) = self.runtime.ibon {
            usart.ctrla().modify(|_, w| w.ibon().bit(ibon));
        }
        if let Some(sbmode) = self.runtime.sbmode {
            usart.ctrlb().modify(|_, w| w.sbmode().bit(sbmode));
        }
        if let Some(pmode) = self.runtime.pmode {
            usart.ctrlb().modify(|_, w| w.pmode().bit(pmode));
        }
        if let Some(sfde) = self.runtime.sfde {
            usart.ctrlb().modify(|_, w| w.sfde().bit(sfde));
        }
        if let Some(colden) = self.runtime.colden {
            usart.ctrlb().modify(|_, w| w.colden().bit(colden));
        }
        if let Some(threshold) = self.runtime.start_of_frame_threshold {
            usart
                .rxpl()
                .write(|w| unsafe { w.rxpl().bits(threshold) });
        }

        let sampr = self.runtime.sampr.unwrap_or(0);
        usart.ctrla().modify(|_, w| unsafe { w.sampr().bits(sampr) });

        if let Some(sampa) = self.runtime.sampa {
            usart.ctrla().modify(|_, w| unsafe { w.sampa().bits(sampa) });
        }

        let oversampling = match sampr {
            0 | 1 => 16,
            2 | 3 => 8,
            _ => 16,
        };
        let baud_rate = self.runtime.baud.unwrap_or(115_200);
        let baud_reg = calculate_baud_asynchronous_arithm(baud_rate, core_clock_hz, oversampling);
        usart
            .baud_usartfp_mode()
            .write(|w| unsafe { w.baud().bits(baud_reg) });

        #[cfg(any(feature = "samd51g", feature = "samd51j", feature = "samd51n", feature = "samd51p", feature = "same51g", feature = "same51j", feature = "same51n", feature = "same53j", feature = "same53n", feature = "same54n", feature = "same54p"))]
        {
            if let Some(gtime) = self.runtime.gtime {
                usart.ctrlc().modify(|_, w| unsafe { w.gtime().bits(gtime) });
            }
            if let Some(brklen) = self.runtime.brklen {
                usart.ctrlc().modify(|_, w| unsafe { w.brklen().bits(brklen) });
            }
            if let Some(hdrdly) = self.runtime.hdrdly {
                usart.ctrlc().modify(|_, w| unsafe { w.hdrdly().bits(hdrdly) });
            }
            if let Some(maxiter) = self.runtime.maxiter {
                usart.ctrlc().modify(|_, w| unsafe { w.maxiter().bits(maxiter) });
            }
            if let Some(inack) = self.runtime.inack {
                usart.ctrlc().modify(|_, w| w.inack().bit(inack));
            }
            if let Some(dsnack) = self.runtime.dsnack {
                usart.ctrlc().modify(|_, w| w.dsnack().bit(dsnack));
            }
        }

        usart.ctrlb().modify(|_, w| {
            w.rxen().bit(<Pads::Capability as CapabilityFlags>::RX);
            w.txen().bit(<Pads::Capability as CapabilityFlags>::TX)
        });
        while usart.syncbusy().read().ctrlb().bit_is_set() {}

        usart.ctrla().modify(|_, w| w.enable().set_bit());
        while usart.syncbusy().read().enable().bit_is_set() {}

        BasicUsart {
            sercom,
            resources: self.resources,
            runtime: self.runtime,
            _capability: PhantomData,
        }
    }
}

impl<S, Pads, Dma, Irqs>
    UsartConfig<S, super::Disabled, UsartResources<Pads, (), Dma, Irqs>, UsartRuntime>
where
    S: Sercom,
    Pads: ValidPads<Sercom = S>,
    Pads::Capability: CapabilityFlags,
{
    /// Materialize an ephemeral USART config by extracting required hardware
    /// resources from a board-level resource container.
    pub fn enable_from<R>(
        self,
        resources: &mut R,
    ) -> BasicUsart<S, Pads::Capability, Pads, <R as TakeSercomCoreClock<S>>::Clock, Dma, Irqs>
    where
        R: TakeSercom<S> + TakeSercomCoreClock<S> + HasApbClkCtrl,
    {
        let sercom = resources.take_sercom();
        let clock = resources.take_sercom_core_clock();
        let apb = resources.apb_clk_ctrl();
        let config = UsartConfig::new(
            UsartResources {
                pads: self.resources.pads,
                clock,
                dma: self.resources.dma,
                irqs: self.resources.irqs,
            },
            self.runtime,
        );
        config.enable_basic(sercom, apb)
    }

    /// Materialize an ephemeral USART config directly from PAC peripherals
    /// without manually extracting `sercomX` fields.
    pub fn enable_from_pac<Clock>(
        self,
        peripherals: &crate::pac::Peripherals,
        core_clock: Clock,
    ) -> BasicUsart<S, Pads::Capability, Pads, Clock, Dma, Irqs>
    where
        crate::pac::Peripherals: PacTakeSercom<S> + PacApbAccess,
        Clock: SercomCoreClock<S>,
    {
        let sercom = <crate::pac::Peripherals as PacTakeSercom<S>>::pac_take_sercom(peripherals);
        let apb = <crate::pac::Peripherals as PacApbAccess>::pac_apb(peripherals);
        let config = UsartConfig::new(
            UsartResources {
                pads: self.resources.pads,
                clock: core_clock,
                dma: self.resources.dma,
                irqs: self.resources.irqs,
            },
            self.runtime,
        );
        config.enable_basic(sercom, apb)
    }
}

impl<S: Sercom, Capability, Pads, Clock, Dma, Irqs> BasicUsart<S, Capability, Pads, Clock, Dma, Irqs> {
    #[inline]
    pub fn free(self) -> (S, UsartResources<Pads, Clock, Dma, Irqs>, UsartRuntime) {
        (self.sercom, self.resources, self.runtime)
    }
}

impl<S: Sercom, Pads, Clock, Dma, Irqs> BasicUsart<S, Tx, Pads, Clock, Dma, Irqs> {
    #[inline]
    pub fn write_u8(&mut self, byte: u8) {
        let usart = usart(&self.sercom);
        while usart.intflag().read().dre().bit_is_clear() {}
        usart.data().write(|w| unsafe { w.data().bits(byte.into()) });
    }
}

impl<S: Sercom, Pads, Clock, Dma, Irqs> BasicUsart<S, Rx, Pads, Clock, Dma, Irqs> {
    #[inline]
    pub fn read_u8(&mut self) -> u8 {
        let usart = usart(&self.sercom);
        while usart.intflag().read().rxc().bit_is_clear() {}
        usart.data().read().data().bits() as u8
    }
}

impl<S: Sercom, Pads, Clock, Dma, Irqs> BasicUsart<S, Duplex, Pads, Clock, Dma, Irqs> {
    #[inline]
    pub fn write_u8(&mut self, byte: u8) {
        let usart = usart(&self.sercom);
        while usart.intflag().read().dre().bit_is_clear() {}
        usart.data().write(|w| unsafe { w.data().bits(byte.into()) });
    }

    #[inline]
    pub fn read_u8(&mut self) -> u8 {
        let usart = usart(&self.sercom);
        while usart.intflag().read().rxc().bit_is_clear() {}
        usart.data().read().data().bits() as u8
    }
}

/// Deterministic USART auto-selection policy.
///
/// The current policy is "first declared valid mapping".
pub struct FirstValid;

/// Resolve unconfigured GPIO pins into a concrete USART SERCOM + pad set.
pub trait AutoUsartPads<RX, TX, CLK = NoneT, RTS = NoneT, CTS = NoneT> {
    type Sercom: Sercom;
    type Pads: ValidPads<Sercom = Self::Sercom>;

    fn into_pads(rx: RX, tx: TX, clk: CLK, rts: RTS, cts: CTS) -> Self::Pads;
}

#[cfg(any(
    feature = "samd21e",
    feature = "samd21g",
    feature = "samd21j",
    feature = "samd21el",
    feature = "samd21gl"
))]
type AllSercomOrders = mk_hlist!(
    typenum::U0,
    typenum::U1,
    typenum::U2,
    typenum::U3,
    typenum::U4,
    typenum::U5
);

#[cfg(any(
    feature = "samd51g",
    feature = "samd51j",
    feature = "samd51n",
    feature = "samd51p",
    feature = "same51g",
    feature = "same51j",
    feature = "same51n",
    feature = "same53j",
    feature = "same53n",
    feature = "same54n",
    feature = "same54p"
))]
type AllSercomOrders = mk_hlist!(
    typenum::U0,
    typenum::U1,
    typenum::U2,
    typenum::U3,
    typenum::U4,
    typenum::U5,
    typenum::U6,
    typenum::U7
);

#[cfg(any(
    feature = "samd21e",
    feature = "samd21g",
    feature = "samd21j",
    feature = "samd21el",
    feature = "samd21gl",
    feature = "samd51g",
    feature = "samd51j",
    feature = "samd51n",
    feature = "samd51p",
    feature = "same51g",
    feature = "same51j",
    feature = "same51n",
    feature = "same53j",
    feature = "same53n",
    feature = "same54n",
    feature = "same54p"
))]
pub trait Head {
    type Head;
}

#[cfg(any(
    feature = "samd21e",
    feature = "samd21g",
    feature = "samd21j",
    feature = "samd21el",
    feature = "samd21gl",
    feature = "samd51g",
    feature = "samd51j",
    feature = "samd51n",
    feature = "samd51p",
    feature = "same51g",
    feature = "same51j",
    feature = "same51n",
    feature = "same53j",
    feature = "same53n",
    feature = "same54n",
    feature = "same54p"
))]
impl<H, T: HList> Head for HCons<H, T> {
    type Head = H;
}

#[cfg(any(
    feature = "samd21e",
    feature = "samd21g",
    feature = "samd21j",
    feature = "samd21el",
    feature = "samd21gl",
    feature = "samd51g",
    feature = "samd51j",
    feature = "samd51n",
    feature = "samd51p",
    feature = "same51g",
    feature = "same51j",
    feature = "same51n",
    feature = "same53j",
    feature = "same53n",
    feature = "same54n",
    feature = "same54p"
))]
pub trait PinIdSercoms {
    type Sercoms: HList;
}

#[cfg(any(
    feature = "samd21e",
    feature = "samd21g",
    feature = "samd21j",
    feature = "samd21el",
    feature = "samd21gl",
    feature = "samd51g",
    feature = "samd51j",
    feature = "samd51n",
    feature = "samd51p",
    feature = "same51g",
    feature = "same51j",
    feature = "same51n",
    feature = "same53j",
    feature = "same53n",
    feature = "same54n",
    feature = "same54p"
))]
impl<I: PinId + PinSercoms> PinIdSercoms for I {
    type Sercoms = I::Sercoms;
}

#[cfg(any(
    feature = "samd21e",
    feature = "samd21g",
    feature = "samd21j",
    feature = "samd21el",
    feature = "samd21gl",
    feature = "samd51g",
    feature = "samd51j",
    feature = "samd51n",
    feature = "samd51p",
    feature = "same51g",
    feature = "same51j",
    feature = "same51n",
    feature = "same53j",
    feature = "same53n",
    feature = "same54n",
    feature = "same54p"
))]
impl PinIdSercoms for NoneT {
    type Sercoms = AllSercomOrders;
}

#[cfg(any(
    feature = "samd21e",
    feature = "samd21g",
    feature = "samd21j",
    feature = "samd21el",
    feature = "samd21gl",
    feature = "samd51g",
    feature = "samd51j",
    feature = "samd51n",
    feature = "samd51p",
    feature = "same51g",
    feature = "same51j",
    feature = "same51n",
    feature = "same53j",
    feature = "same53n",
    feature = "same54n",
    feature = "same54p"
))]
pub trait ResolveUsartSercom<RX, TX, CLK, RTS, CTS> {
    type Sercom: Sercom + SercomOrder;
}

#[cfg(any(
    feature = "samd21e",
    feature = "samd21g",
    feature = "samd21j",
    feature = "samd21el",
    feature = "samd21gl",
    feature = "samd51g",
    feature = "samd51j",
    feature = "samd51n",
    feature = "samd51p",
    feature = "same51g",
    feature = "same51j",
    feature = "same51n",
    feature = "same53j",
    feature = "same53n",
    feature = "same54n",
    feature = "same54p"
))]
pub trait SercomFromOrder {
    type Sercom: Sercom + SercomOrder;
}

#[cfg(any(feature = "samd21e", feature = "samd21g", feature = "samd21j", feature = "samd21el", feature = "samd21gl", feature = "samd51g", feature = "samd51j", feature = "samd51n", feature = "samd51p", feature = "same51g", feature = "same51j", feature = "same51n", feature = "same53j", feature = "same53n", feature = "same54n", feature = "same54p"))]
impl SercomFromOrder for typenum::U0 {
    type Sercom = crate::sercom_v2::Sercom0;
}
#[cfg(any(feature = "samd21e", feature = "samd21g", feature = "samd21j", feature = "samd21el", feature = "samd21gl", feature = "samd51g", feature = "samd51j", feature = "samd51n", feature = "samd51p", feature = "same51g", feature = "same51j", feature = "same51n", feature = "same53j", feature = "same53n", feature = "same54n", feature = "same54p"))]
impl SercomFromOrder for typenum::U1 {
    type Sercom = crate::sercom_v2::Sercom1;
}
#[cfg(any(feature = "samd21e", feature = "samd21g", feature = "samd21j", feature = "samd21el", feature = "samd21gl", feature = "samd51g", feature = "samd51j", feature = "samd51n", feature = "samd51p", feature = "same51g", feature = "same51j", feature = "same51n", feature = "same53j", feature = "same53n", feature = "same54n", feature = "same54p"))]
impl SercomFromOrder for typenum::U2 {
    type Sercom = crate::sercom_v2::Sercom2;
}
#[cfg(any(feature = "samd21e", feature = "samd21g", feature = "samd21j", feature = "samd21el", feature = "samd21gl", feature = "samd51g", feature = "samd51j", feature = "samd51n", feature = "samd51p", feature = "same51g", feature = "same51j", feature = "same51n", feature = "same53j", feature = "same53n", feature = "same54n", feature = "same54p"))]
impl SercomFromOrder for typenum::U3 {
    type Sercom = crate::sercom_v2::Sercom3;
}
#[cfg(any(feature = "samd21e", feature = "samd21g", feature = "samd21j", feature = "samd21el", feature = "samd21gl", feature = "samd51g", feature = "samd51j", feature = "samd51n", feature = "samd51p", feature = "same51g", feature = "same51j", feature = "same51n", feature = "same53j", feature = "same53n", feature = "same54n", feature = "same54p"))]
impl SercomFromOrder for typenum::U4 {
    type Sercom = crate::sercom_v2::Sercom4;
}
#[cfg(any(feature = "samd21e", feature = "samd21g", feature = "samd21j", feature = "samd21el", feature = "samd21gl", feature = "samd51g", feature = "samd51j", feature = "samd51n", feature = "samd51p", feature = "same51g", feature = "same51j", feature = "same51n", feature = "same53j", feature = "same53n", feature = "same54n", feature = "same54p"))]
impl SercomFromOrder for typenum::U5 {
    type Sercom = crate::sercom_v2::Sercom5;
}
#[cfg(any(feature = "samd51g", feature = "samd51j", feature = "samd51n", feature = "samd51p", feature = "same51g", feature = "same51j", feature = "same51n", feature = "same53j", feature = "same53n", feature = "same54n", feature = "same54p"))]
#[hal_cfg("sercom6-d5x")]
impl SercomFromOrder for typenum::U6 {
    type Sercom = crate::sercom_v2::Sercom6;
}
#[hal_cfg("sercom7-d5x")]
impl SercomFromOrder for typenum::U7 {
    type Sercom = crate::sercom_v2::Sercom7;
}

#[cfg(any(
    feature = "samd21e",
    feature = "samd21g",
    feature = "samd21j",
    feature = "samd21el",
    feature = "samd21gl",
    feature = "samd51g",
    feature = "samd51j",
    feature = "samd51n",
    feature = "samd51p",
    feature = "same51g",
    feature = "same51j",
    feature = "same51n",
    feature = "same53j",
    feature = "same53n",
    feature = "same54n",
    feature = "same54p"
))]
impl<RX, TX, CLK, RTS, CTS> ResolveUsartSercom<RX, TX, CLK, RTS, CTS> for FirstValid
where
    RX: PinIdSercoms,
    TX: PinIdSercoms,
    CLK: PinIdSercoms,
    RTS: PinIdSercoms,
    CTS: PinIdSercoms,
    AllSercomOrders: IntersectUnchecked<RX::Sercoms>,
    <AllSercomOrders as IntersectUnchecked<RX::Sercoms>>::Output: IntersectUnchecked<TX::Sercoms>,
    <<AllSercomOrders as IntersectUnchecked<RX::Sercoms>>::Output as IntersectUnchecked<TX::Sercoms>>::Output:
        IntersectUnchecked<CLK::Sercoms>,
    <<<AllSercomOrders as IntersectUnchecked<RX::Sercoms>>::Output as IntersectUnchecked<TX::Sercoms>>::Output as IntersectUnchecked<CLK::Sercoms>>::Output:
        IntersectUnchecked<RTS::Sercoms>,
    <<<<AllSercomOrders as IntersectUnchecked<RX::Sercoms>>::Output as IntersectUnchecked<TX::Sercoms>>::Output as IntersectUnchecked<CLK::Sercoms>>::Output as IntersectUnchecked<RTS::Sercoms>>::Output:
        IntersectUnchecked<CTS::Sercoms>,
    <<<<<AllSercomOrders as IntersectUnchecked<RX::Sercoms>>::Output as IntersectUnchecked<TX::Sercoms>>::Output as IntersectUnchecked<CLK::Sercoms>>::Output as IntersectUnchecked<RTS::Sercoms>>::Output as IntersectUnchecked<CTS::Sercoms>>::Output:
        NonEmptyHList + Head,
    <<<<<<AllSercomOrders as IntersectUnchecked<RX::Sercoms>>::Output as IntersectUnchecked<TX::Sercoms>>::Output as IntersectUnchecked<CLK::Sercoms>>::Output as IntersectUnchecked<RTS::Sercoms>>::Output as IntersectUnchecked<CTS::Sercoms>>::Output as Head>::Head:
        SercomFromOrder,
{
    type Sercom = <<<<<<<AllSercomOrders as IntersectUnchecked<RX::Sercoms>>::Output as IntersectUnchecked<
        TX::Sercoms,
    >>::Output as IntersectUnchecked<CLK::Sercoms>>::Output as IntersectUnchecked<RTS::Sercoms>>::Output as IntersectUnchecked<
        CTS::Sercoms,
    >>::Output as Head>::Head as SercomFromOrder>::Sercom;
}


/// Builder accepting unconfigured GPIO pins and producing typed USART config.
#[derive(Debug)]
pub struct AutoUsartBuilder<RX = NoneT, TX = NoneT, CLK = NoneT, RTS = NoneT, CTS = NoneT> {
    rx: RX,
    tx: TX,
    clk: CLK,
    rts: RTS,
    cts: CTS,
    runtime: UsartRuntime,
}

impl Default for AutoUsartBuilder {
    fn default() -> Self {
        Self {
            rx: NoneT,
            tx: NoneT,
            clk: NoneT,
            rts: NoneT,
            cts: NoneT,
            runtime: UsartRuntime::default(),
        }
    }
}

impl AutoUsartBuilder {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Usart {
    #[inline]
    pub fn default() -> AutoUsartBuilder {
        AutoUsartBuilder::default()
    }
}

impl<TX, CLK, RTS, CTS> AutoUsartBuilder<NoneT, TX, CLK, RTS, CTS> {
    pub fn rx<I: PinId, M: PinMode>(
        self,
        pin: Pin<I, M>,
    ) -> AutoUsartBuilder<Pin<I, M>, TX, CLK, RTS, CTS> {
        AutoUsartBuilder {
            rx: pin,
            tx: self.tx,
            clk: self.clk,
            rts: self.rts,
            cts: self.cts,
            runtime: self.runtime,
        }
    }
}

impl<RX, CLK, RTS, CTS> AutoUsartBuilder<RX, NoneT, CLK, RTS, CTS> {
    pub fn tx<I: PinId, M: PinMode>(
        self,
        pin: Pin<I, M>,
    ) -> AutoUsartBuilder<RX, Pin<I, M>, CLK, RTS, CTS> {
        AutoUsartBuilder {
            rx: self.rx,
            tx: pin,
            clk: self.clk,
            rts: self.rts,
            cts: self.cts,
            runtime: self.runtime,
        }
    }
}

impl<RX, TX, RTS, CTS> AutoUsartBuilder<RX, TX, NoneT, RTS, CTS> {
    pub fn clk<I: PinId, M: PinMode>(
        self,
        pin: Pin<I, M>,
    ) -> AutoUsartBuilder<RX, TX, Pin<I, M>, RTS, CTS> {
        AutoUsartBuilder {
            rx: self.rx,
            tx: self.tx,
            clk: pin,
            rts: self.rts,
            cts: self.cts,
            runtime: self.runtime,
        }
    }
}

impl<RX, TX, CLK, CTS> AutoUsartBuilder<RX, TX, CLK, NoneT, CTS> {
    pub fn rts<I: PinId, M: PinMode>(
        self,
        pin: Pin<I, M>,
    ) -> AutoUsartBuilder<RX, TX, CLK, Pin<I, M>, CTS> {
        AutoUsartBuilder {
            rx: self.rx,
            tx: self.tx,
            clk: self.clk,
            rts: pin,
            cts: self.cts,
            runtime: self.runtime,
        }
    }
}

impl<RX, TX, CLK, RTS> AutoUsartBuilder<RX, TX, CLK, RTS, NoneT> {
    pub fn cts<I: PinId, M: PinMode>(
        self,
        pin: Pin<I, M>,
    ) -> AutoUsartBuilder<RX, TX, CLK, RTS, Pin<I, M>> {
        AutoUsartBuilder {
            rx: self.rx,
            tx: self.tx,
            clk: self.clk,
            rts: self.rts,
            cts: pin,
            runtime: self.runtime,
        }
    }
}

impl<RX, TX, CLK, RTS, CTS> AutoUsartBuilder<RX, TX, CLK, RTS, CTS> {
    pub fn baud(mut self, baud: u32) -> Self {
        self.runtime.baud = Some(baud);
        self
    }

    pub fn dord(mut self, dord: bool) -> Self {
        self.runtime.dord = Some(dord);
        self
    }

    pub fn runstdby(mut self, runstdby: bool) -> Self {
        self.runtime.runstdby = Some(runstdby);
        self
    }
}

impl<RX, TX, CLK, RTS, CTS> AutoUsartBuilder<RX, TX, CLK, RTS, CTS>
where
    FirstValid: AutoUsartPads<RX, TX, CLK, RTS, CTS>,
{
    /// Resolve pins with the default "first valid mapping" policy and keep
    /// clock ownership external until `enable_from`.
    pub fn to_config(self) -> UsartConfig<
        <FirstValid as AutoUsartPads<RX, TX, CLK, RTS, CTS>>::Sercom,
        super::Disabled,
        UsartResources<<FirstValid as AutoUsartPads<RX, TX, CLK, RTS, CTS>>::Pads, ()>,
        UsartRuntime,
    > {
        let pads = <FirstValid as AutoUsartPads<RX, TX, CLK, RTS, CTS>>::into_pads(
            self.rx,
            self.tx,
            self.clk,
            self.rts,
            self.cts,
        );
        let resources = UsartResources {
            pads,
            clock: (),
            dma: (),
            irqs: (),
        };
        UsartConfig::new(resources, self.runtime)
    }

    /// Resolve pins with the default "first valid mapping" policy.
    pub fn to_config_first(
        self,
        clock_hz: u32,
    ) -> UsartConfig<
        <FirstValid as AutoUsartPads<RX, TX, CLK, RTS, CTS>>::Sercom,
        super::Disabled,
        UsartResources<<FirstValid as AutoUsartPads<RX, TX, CLK, RTS, CTS>>::Pads, u32>,
        UsartRuntime,
    > {
        let pads = <FirstValid as AutoUsartPads<RX, TX, CLK, RTS, CTS>>::into_pads(
            self.rx,
            self.tx,
            self.clk,
            self.rts,
            self.cts,
        );
        let resources = UsartResources {
            pads,
            clock: clock_hz,
            dma: (),
            irqs: (),
        };
        UsartConfig::new(resources, self.runtime)
    }
}

#[cfg(any(
    feature = "samd21e",
    feature = "samd21g",
    feature = "samd21j",
    feature = "samd21el",
    feature = "samd21gl",
    feature = "samd51g",
    feature = "samd51j",
    feature = "samd51n",
    feature = "samd51p",
    feature = "same51g",
    feature = "same51j",
    feature = "same51n",
    feature = "same53j",
    feature = "same53n",
    feature = "same54n",
    feature = "same54p"
))]
type AutoSercom<RX, TX, CLK, RTS, CTS> = <FirstValid as ResolveUsartSercom<RX, TX, CLK, RTS, CTS>>::Sercom;

#[cfg(any(
    feature = "samd21e",
    feature = "samd21g",
    feature = "samd21j",
    feature = "samd21el",
    feature = "samd21gl",
    feature = "samd51g",
    feature = "samd51j",
    feature = "samd51n",
    feature = "samd51p",
    feature = "same51g",
    feature = "same51j",
    feature = "same51n",
    feature = "same53j",
    feature = "same53n",
    feature = "same54n",
    feature = "same54p"
))]
type AutoPad<RX, TX, CLK, RTS, CTS, I> = crate::sercom_v2::Pad<AutoSercom<RX, TX, CLK, RTS, CTS>, I>;

#[cfg(any(
    feature = "samd21e",
    feature = "samd21g",
    feature = "samd21j",
    feature = "samd21el",
    feature = "samd21gl",
    feature = "samd51g",
    feature = "samd51j",
    feature = "samd51n",
    feature = "samd51p",
    feature = "same51g",
    feature = "same51j",
    feature = "same51n",
    feature = "same53j",
    feature = "same53n",
    feature = "same54n",
    feature = "same54p"
))]
macro_rules! impl_auto_usart_case {
    (
        impl<$($gen:ident),+> AutoUsartPads<$rx_ty:ty, $tx_ty:ty, $clk_ty:ty, $rts_ty:ty, $cts_ty:ty>
        where
            resolve = ($rx_id:ty, $tx_id:ty, $clk_id:ty, $rts_id:ty, $cts_id:ty);
            getpad = [$($pid:ty),+];
            pinmode = [$($pmode:ty),+];
            rxpo = $rxpo:ty;
            txpo = $txpo:ty;
            pads = $pads:ty;
            into = |$rxv:ident, $txv:ident, $clkv:ident, $rtsv:ident, $ctsv:ident| $body:block
    ) => {
        impl<$($gen),+> AutoUsartPads<$rx_ty, $tx_ty, $clk_ty, $rts_ty, $cts_ty> for FirstValid
        where
            FirstValid: ResolveUsartSercom<$rx_id, $tx_id, $clk_id, $rts_id, $cts_id>,
            $($pid: PinId + PinSercoms,)+
            $($pmode: PinMode,)+
            $($pid: crate::sercom_v2::GetPad<AutoSercom<$rx_id, $tx_id, $clk_id, $rts_id, $cts_id>> ,)+
            $(AutoPad<$rx_id, $tx_id, $clk_id, $rts_id, $cts_id, $pid>: IsPad<Sercom = AutoSercom<$rx_id, $tx_id, $clk_id, $rts_id, $cts_id>>,)+
            $rxpo: Rxpo,
            $txpo: Txpo,
        {
            type Sercom = AutoSercom<$rx_id, $tx_id, $clk_id, $rts_id, $cts_id>;
            type Pads = $pads;
            fn into_pads($rxv: $rx_ty, $txv: $tx_ty, $clkv: $clk_ty, $rtsv: $rts_ty, $ctsv: $cts_ty) -> Self::Pads {
                $body
            }
        }
    };
}

#[cfg(any(
    feature = "samd21e",
    feature = "samd21g",
    feature = "samd21j",
    feature = "samd21el",
    feature = "samd21gl",
    feature = "samd51g",
    feature = "samd51j",
    feature = "samd51n",
    feature = "samd51p",
    feature = "same51g",
    feature = "same51j",
    feature = "same51n",
    feature = "same53j",
    feature = "same53n",
    feature = "same54n",
    feature = "same54p"
))]
impl_auto_usart_case! {
    impl<RXI, RXM, TXI, TXM> AutoUsartPads<Pin<RXI, RXM>, Pin<TXI, TXM>, NoneT, NoneT, NoneT>
    where
        resolve = (RXI, TXI, NoneT, NoneT, NoneT);
        getpad = [RXI, TXI];
        pinmode = [RXM, TXM];
        rxpo = <AutoPad<RXI, TXI, NoneT, NoneT, NoneT, RXI> as IsPad>::PadNum;
        txpo = (
            <AutoPad<RXI, TXI, NoneT, NoneT, NoneT, TXI> as IsPad>::PadNum,
            NoneT, NoneT, NoneT
        );
        pads = InternalPads<
            pads::Pads<
                AutoPad<RXI, TXI, NoneT, NoneT, NoneT, RXI>,
                AutoPad<RXI, TXI, NoneT, NoneT, NoneT, TXI>,
                NoneT,
                NoneT
            >,
            Roles<
                AutoPad<RXI, TXI, NoneT, NoneT, NoneT, RXI>,
                AutoPad<RXI, TXI, NoneT, NoneT, NoneT, TXI>,
                NoneT, NoneT, NoneT
            >
        >;
        into = |rx, tx, _clk, _rts, _cts| {
            InternalPads {
                pads: pads::Pads(
                    rx.into_mode::<crate::sercom_v2::PadMode<
                        AutoSercom<RXI, TXI, NoneT, NoneT, NoneT>,
                        RXI,
                    >>(),
                    tx.into_mode::<crate::sercom_v2::PadMode<
                        AutoSercom<RXI, TXI, NoneT, NoneT, NoneT>,
                        TXI,
                    >>(),
                    NoneT,
                    NoneT,
                ),
                roles: Roles::default(),
            }
        }
}

#[cfg(any(
    feature = "samd21e",
    feature = "samd21g",
    feature = "samd21j",
    feature = "samd21el",
    feature = "samd21gl",
    feature = "samd51g",
    feature = "samd51j",
    feature = "samd51n",
    feature = "samd51p",
    feature = "same51g",
    feature = "same51j",
    feature = "same51n",
    feature = "same53j",
    feature = "same53n",
    feature = "same54n",
    feature = "same54p"
))]
impl_auto_usart_case! {
    impl<RXI, RXM, TXI, TXM, CTSI, CTSM> AutoUsartPads<Pin<RXI, RXM>, Pin<TXI, TXM>, NoneT, NoneT, Pin<CTSI, CTSM>>
    where
        resolve = (RXI, TXI, NoneT, NoneT, CTSI);
        getpad = [RXI, TXI, CTSI];
        pinmode = [RXM, TXM, CTSM];
        
        rxpo = <crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, NoneT, CTSI>>::Sercom, RXI> as IsPad>::PadNum;
        txpo = (
            <crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, NoneT, CTSI>>::Sercom, TXI> as IsPad>::PadNum,
            NoneT,
            NoneT,
            <crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, NoneT, CTSI>>::Sercom, CTSI> as IsPad>::PadNum
        );
        pads = InternalPads<
            pads::Pads<
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, NoneT, CTSI>>::Sercom, RXI>,
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, NoneT, CTSI>>::Sercom, TXI>,
                NoneT,
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, NoneT, CTSI>>::Sercom, CTSI>
            >,
            Roles<
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, NoneT, CTSI>>::Sercom, RXI>,
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, NoneT, CTSI>>::Sercom, TXI>,
                NoneT,
                NoneT,
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, NoneT, CTSI>>::Sercom, CTSI>
            >
        >;
        into = |rx, tx, _clk, _rts, cts| {
            InternalPads {
                pads: pads::Pads(
                    rx.into_mode::<crate::sercom_v2::PadMode<
                        <FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, NoneT, CTSI>>::Sercom,
                        RXI,
                    >>(),
                    tx.into_mode::<crate::sercom_v2::PadMode<
                        <FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, NoneT, CTSI>>::Sercom,
                        TXI,
                    >>(),
                    NoneT,
                    cts.into_mode::<crate::sercom_v2::PadMode<
                        <FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, NoneT, CTSI>>::Sercom,
                        CTSI,
                    >>(),
                ),
                roles: Roles::default(),
            }
        }
}

#[cfg(any(
    feature = "samd21e",
    feature = "samd21g",
    feature = "samd21j",
    feature = "samd21el",
    feature = "samd21gl",
    feature = "samd51g",
    feature = "samd51j",
    feature = "samd51n",
    feature = "samd51p",
    feature = "same51g",
    feature = "same51j",
    feature = "same51n",
    feature = "same53j",
    feature = "same53n",
    feature = "same54n",
    feature = "same54p"
))]
impl_auto_usart_case! {
    impl<RXI, RXM, TXI, TXM, CLKI, CLKM, RTSI, RTSM> AutoUsartPads<Pin<RXI, RXM>, Pin<TXI, TXM>, Pin<CLKI, CLKM>, Pin<RTSI, RTSM>, NoneT>
    where
        resolve = (RXI, TXI, CLKI, RTSI, NoneT);
        getpad = [RXI, TXI, CLKI, RTSI];
        pinmode = [RXM, TXM, CLKM, RTSM];
        
        rxpo = <crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, RTSI, NoneT>>::Sercom, RXI> as IsPad>::PadNum;
        txpo = (
            <crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, RTSI, NoneT>>::Sercom, TXI> as IsPad>::PadNum,
            <crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, RTSI, NoneT>>::Sercom, CLKI> as IsPad>::PadNum,
            <crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, RTSI, NoneT>>::Sercom, RTSI> as IsPad>::PadNum,
            NoneT
        );
        pads = InternalPads<
            pads::Pads<
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, RTSI, NoneT>>::Sercom, RXI>,
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, RTSI, NoneT>>::Sercom, TXI>,
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, RTSI, NoneT>>::Sercom, CLKI>,
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, RTSI, NoneT>>::Sercom, RTSI>
            >,
            Roles<
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, RTSI, NoneT>>::Sercom, RXI>,
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, RTSI, NoneT>>::Sercom, TXI>,
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, RTSI, NoneT>>::Sercom, CLKI>,
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, RTSI, NoneT>>::Sercom, RTSI>,
                NoneT
            >
        >;
        into = |rx, tx, clk, rts, _cts| {
            InternalPads {
                pads: pads::Pads(
                    rx.into_mode::<crate::sercom_v2::PadMode<
                        <FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, RTSI, NoneT>>::Sercom,
                        RXI,
                    >>(),
                    tx.into_mode::<crate::sercom_v2::PadMode<
                        <FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, RTSI, NoneT>>::Sercom,
                        TXI,
                    >>(),
                    clk.into_mode::<crate::sercom_v2::PadMode<
                        <FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, RTSI, NoneT>>::Sercom,
                        CLKI,
                    >>(),
                    rts.into_mode::<crate::sercom_v2::PadMode<
                        <FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, RTSI, NoneT>>::Sercom,
                        RTSI,
                    >>(),
                ),
                roles: Roles::default(),
            }
        }
}

#[cfg(any(
    feature = "samd21e",
    feature = "samd21g",
    feature = "samd21j",
    feature = "samd21el",
    feature = "samd21gl",
    feature = "samd51g",
    feature = "samd51j",
    feature = "samd51n",
    feature = "samd51p",
    feature = "same51g",
    feature = "same51j",
    feature = "same51n",
    feature = "same53j",
    feature = "same53n",
    feature = "same54n",
    feature = "same54p"
))]
impl_auto_usart_case! {
    impl<RXI, RXM, TXI, TXM, CLKI, CLKM, CTSI, CTSM> AutoUsartPads<Pin<RXI, RXM>, Pin<TXI, TXM>, Pin<CLKI, CLKM>, NoneT, Pin<CTSI, CTSM>>
    where
        resolve = (RXI, TXI, CLKI, NoneT, CTSI);
        getpad = [RXI, TXI, CLKI, CTSI];
        pinmode = [RXM, TXM, CLKM, CTSM];
        
        rxpo = <crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, NoneT, CTSI>>::Sercom, RXI> as IsPad>::PadNum;
        txpo = (
            <crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, NoneT, CTSI>>::Sercom, TXI> as IsPad>::PadNum,
            <crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, NoneT, CTSI>>::Sercom, CLKI> as IsPad>::PadNum,
            NoneT,
            <crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, NoneT, CTSI>>::Sercom, CTSI> as IsPad>::PadNum
        );
        pads = InternalPads<
            pads::Pads<
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, NoneT, CTSI>>::Sercom, RXI>,
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, NoneT, CTSI>>::Sercom, TXI>,
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, NoneT, CTSI>>::Sercom, CLKI>,
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, NoneT, CTSI>>::Sercom, CTSI>
            >,
            Roles<
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, NoneT, CTSI>>::Sercom, RXI>,
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, NoneT, CTSI>>::Sercom, TXI>,
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, NoneT, CTSI>>::Sercom, CLKI>,
                NoneT,
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, NoneT, CTSI>>::Sercom, CTSI>
            >
        >;
        into = |rx, tx, clk, _rts, cts| {
            InternalPads {
                pads: pads::Pads(
                    rx.into_mode::<crate::sercom_v2::PadMode<
                        <FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, NoneT, CTSI>>::Sercom,
                        RXI,
                    >>(),
                    tx.into_mode::<crate::sercom_v2::PadMode<
                        <FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, NoneT, CTSI>>::Sercom,
                        TXI,
                    >>(),
                    clk.into_mode::<crate::sercom_v2::PadMode<
                        <FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, NoneT, CTSI>>::Sercom,
                        CLKI,
                    >>(),
                    cts.into_mode::<crate::sercom_v2::PadMode<
                        <FirstValid as ResolveUsartSercom<RXI, TXI, CLKI, NoneT, CTSI>>::Sercom,
                        CTSI,
                    >>(),
                ),
                roles: Roles::default(),
            }
        }
}

#[cfg(any(
    feature = "samd21e",
    feature = "samd21g",
    feature = "samd21j",
    feature = "samd21el",
    feature = "samd21gl",
    feature = "samd51g",
    feature = "samd51j",
    feature = "samd51n",
    feature = "samd51p",
    feature = "same51g",
    feature = "same51j",
    feature = "same51n",
    feature = "same53j",
    feature = "same53n",
    feature = "same54n",
    feature = "same54p"
))]
impl_auto_usart_case! {
    impl<RXI, RXM, TXI, TXM, RTSI, RTSM, CTSI, CTSM> AutoUsartPads<Pin<RXI, RXM>, Pin<TXI, TXM>, NoneT, Pin<RTSI, RTSM>, Pin<CTSI, CTSM>>
    where
        resolve = (RXI, TXI, NoneT, RTSI, CTSI);
        getpad = [RXI, TXI, RTSI, CTSI];
        pinmode = [RXM, TXM, RTSM, CTSM];
        
        rxpo = <crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, RTSI, CTSI>>::Sercom, RXI> as IsPad>::PadNum;
        txpo = (
            <crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, RTSI, CTSI>>::Sercom, TXI> as IsPad>::PadNum,
            NoneT,
            <crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, RTSI, CTSI>>::Sercom, RTSI> as IsPad>::PadNum,
            <crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, RTSI, CTSI>>::Sercom, CTSI> as IsPad>::PadNum
        );
        pads = InternalPads<
            pads::Pads<
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, RTSI, CTSI>>::Sercom, RXI>,
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, RTSI, CTSI>>::Sercom, TXI>,
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, RTSI, CTSI>>::Sercom, RTSI>,
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, RTSI, CTSI>>::Sercom, CTSI>
            >,
            Roles<
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, RTSI, CTSI>>::Sercom, RXI>,
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, RTSI, CTSI>>::Sercom, TXI>,
                NoneT,
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, RTSI, CTSI>>::Sercom, RTSI>,
                crate::sercom_v2::Pad<<FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, RTSI, CTSI>>::Sercom, CTSI>
            >
        >;
        into = |rx, tx, _clk, rts, cts| {
            InternalPads {
                pads: pads::Pads(
                    rx.into_mode::<crate::sercom_v2::PadMode<
                        <FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, RTSI, CTSI>>::Sercom,
                        RXI,
                    >>(),
                    tx.into_mode::<crate::sercom_v2::PadMode<
                        <FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, RTSI, CTSI>>::Sercom,
                        TXI,
                    >>(),
                    rts.into_mode::<crate::sercom_v2::PadMode<
                        <FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, RTSI, CTSI>>::Sercom,
                        RTSI,
                    >>(),
                    cts.into_mode::<crate::sercom_v2::PadMode<
                        <FirstValid as ResolveUsartSercom<RXI, TXI, NoneT, RTSI, CTSI>>::Sercom,
                        CTSI,
                    >>(),
                ),
                roles: Roles::default(),
            }
        }
}

#[cfg(any(
    feature = "samd21e",
    feature = "samd21g",
    feature = "samd21j",
    feature = "samd21el",
    feature = "samd21gl",
    feature = "samd51g",
    feature = "samd51j",
    feature = "samd51n",
    feature = "samd51p",
    feature = "same51g",
    feature = "same51j",
    feature = "same51n",
    feature = "same53j",
    feature = "same53n",
    feature = "same54n",
    feature = "same54p"
))]
impl_auto_usart_case! {
    impl<RXI, RXM, TXI, TXM, CLKI, CLKM> AutoUsartPads<Pin<RXI, RXM>, Pin<TXI, TXM>, Pin<CLKI, CLKM>, NoneT, NoneT>
    where
        resolve = (RXI, TXI, CLKI, NoneT, NoneT);
        getpad = [RXI, TXI, CLKI];
        pinmode = [RXM, TXM, CLKM];
        rxpo = <AutoPad<RXI, TXI, CLKI, NoneT, NoneT, RXI> as IsPad>::PadNum;
        txpo = (
            <AutoPad<RXI, TXI, CLKI, NoneT, NoneT, TXI> as IsPad>::PadNum,
            <AutoPad<RXI, TXI, CLKI, NoneT, NoneT, CLKI> as IsPad>::PadNum,
            NoneT,
            NoneT
        );
        pads = InternalPads<
            pads::Pads<
                AutoPad<RXI, TXI, CLKI, NoneT, NoneT, RXI>,
                AutoPad<RXI, TXI, CLKI, NoneT, NoneT, TXI>,
                AutoPad<RXI, TXI, CLKI, NoneT, NoneT, CLKI>,
                NoneT
            >,
            Roles<
                AutoPad<RXI, TXI, CLKI, NoneT, NoneT, RXI>,
                AutoPad<RXI, TXI, CLKI, NoneT, NoneT, TXI>,
                AutoPad<RXI, TXI, CLKI, NoneT, NoneT, CLKI>,
                NoneT,
                NoneT
            >
        >;
        into = |rx, tx, clk, _rts, _cts| {
            InternalPads {
                pads: pads::Pads(
                    rx.into_mode::<crate::sercom_v2::PadMode<
                        AutoSercom<RXI, TXI, CLKI, NoneT, NoneT>,
                        RXI,
                    >>(),
                    tx.into_mode::<crate::sercom_v2::PadMode<
                        AutoSercom<RXI, TXI, CLKI, NoneT, NoneT>,
                        TXI,
                    >>(),
                    clk.into_mode::<crate::sercom_v2::PadMode<
                        AutoSercom<RXI, TXI, CLKI, NoneT, NoneT>,
                        CLKI,
                    >>(),
                    NoneT,
                ),
                roles: Roles::default(),
            }
        }
}

#[cfg(any(
    feature = "samd51g",
    feature = "samd51j",
    feature = "samd51n",
    feature = "samd51p",
    feature = "same51g",
    feature = "same51j",
    feature = "same51n",
    feature = "same53j",
    feature = "same53n",
    feature = "same54n",
    feature = "same54p"
))]
impl<RXM: PinMode, TXM: PinMode, RTSM: PinMode>
    AutoUsartPads<
        Pin<crate::gpio::PA22, RXM>,
        Pin<crate::gpio::PA23, TXM>,
        NoneT,
        Pin<crate::gpio::PA24, RTSM>,
        NoneT,
    > for FirstValid
{
    type Sercom = crate::sercom_v2::Sercom5;
    type Pads = Pads<
        crate::sercom_v2::Pad<crate::sercom_v2::Sercom5, crate::gpio::PA22>,
        crate::sercom_v2::Pad<crate::sercom_v2::Sercom5, crate::gpio::PA23>,
        crate::sercom_v2::Pad<crate::sercom_v2::Sercom5, crate::gpio::PA24>,
    >;

    fn into_pads(
        rx: Pin<crate::gpio::PA22, RXM>,
        tx: Pin<crate::gpio::PA23, TXM>,
        _: NoneT,
        rts: Pin<crate::gpio::PA24, RTSM>,
        _: NoneT,
    ) -> Self::Pads {
        InternalPads::default()
            .rx(rx.into_mode::<crate::sercom_v2::PadMode<crate::sercom_v2::Sercom5, crate::gpio::PA22>>())
            .tx(tx.into_mode::<crate::sercom_v2::PadMode<crate::sercom_v2::Sercom5, crate::gpio::PA23>>())
            .rts(rts.into_mode::<crate::sercom_v2::PadMode<crate::sercom_v2::Sercom5, crate::gpio::PA24>>())
    }
}
super::define_sercom_protocol! {
    /// USART protocol marker and baseline containers.
    marker: Usart,
    config: UsartConfig,
    peripheral: UsartPeripheral,
    runtime: UsartRuntime,
    builder: UsartBuilder,
    runtime_fields: {
        dord: bool,
        sampr: u8,
        sampa: u8,
        baud: u32,
        runstdby: bool,
        ibon: bool,
        sbmode: bool,
        pmode: bool,
        sfde: bool,
        colden: bool,
        start_of_frame_threshold: u8,
        gtime: u8,
        brklen: u8,
        hdrdly: u8,
        maxiter: u8,
        inack: bool,
        dsnack: bool,
    },
    fields: [
        ("CTRLA", "SWRST", Function, "Lifecycle reset operation."),
        ("CTRLA", "ENABLE", Generic, "Universal enabled/disabled typestate split."),
        ("CTRLA", "MODE", Generic, "Selects USART external/internal clock mode family."),
        ("CTRLA", "RUNSTDBY", Builder, "Runtime standby behavior, does not change legal surface."),
        ("CTRLA", "IBON", Builder, "Immediate overflow notification policy."),
        ("CTRLA", "FORM", Generic, "Primary semantic frame family: USART, parity, LIN, auto-baud, ISO7816."),
        ("CTRLA", "CMODE", Generic, "Synchronous vs asynchronous semantic mode."),
        ("CTRLA", "CPOL", Builder, "Clock polarity tuning for synchronous mode."),
        ("CTRLA", "DORD", Builder, "Bit order changes encoding, not the abstract word API."),
        ("CTRLA", "SAMPR", Builder, "Oversampling/baud scheme tuning."),
        ("CTRLA", "SAMPA", Builder, "Sampling adjustment tuning."),
        ("CTRLA", "RXPO", Resource, "Register-state justified by owned RX pad routing."),
        ("CTRLA", "TXPO", Resource, "Register-state justified by owned TX/CLK/RTS/CTS routing."),
        ("CTRLA", "SAMPA", Builder, "Sampling phase tuning."),
        ("CTRLA", "FORM", Generic, "Framing choice changes legal higher-level operations."),
        ("CTRLB", "CHSIZE", Generic, "Word size changes the exposed word type."),
        ("CTRLB", "SBMODE", Builder, "Stop-bit count does not change the abstract transaction model."),
        ("CTRLB", "PMODE", Builder, "Parity polarity is runtime-configured framing detail."),
        ("CTRLB", "TXEN", Generic, "TX capability should be reflected in typestate."),
        ("CTRLB", "RXEN", Generic, "RX capability should be reflected in typestate."),
        ("CTRLB", "SFDE", Builder, "Start-of-frame detection tuning."),
        ("CTRLB", "COLDEN", Builder, "Collision detection policy."),
        ("CTRLC", "GTIME", Builder, "Guard time fits sparse runtime builder."),
        ("CTRLC", "BRKLEN", Builder, "Break length only matters in LIN/ISO modes."),
        ("CTRLC", "HDRDLY", Generic, "Only valid in specific LIN header command semantics."),
        ("CTRLC", "INACK", Generic, "Only valid in ISO7816 semantics."),
        ("CTRLC", "DSNACK", Generic, "Only valid in ISO7816 semantics."),
        ("CTRLC", "MAXITER", Builder, "Iteration count is runtime tuning."),
        ("BAUD", "BAUD", Builder, "BAUD value comes from runtime clock/rate calculation."),
        ("DATA", "DATA", Function, "Volatile data path."),
        ("STATUS", "bits", Function, "Volatile status observation."),
        ("INTENSET", "bits", Function, "Interrupt enables are live side effects."),
        ("INTENCLR", "bits", Function, "Interrupt disables are live side effects."),
        ("INTFLAG", "bits", Function, "Interrupt flags are volatile operations."),
        ("SYNCBUSY", "bits", Function, "Synchronization observation only."),
        ("LINCMD", "bits", Function, "Only legal in LIN host mode and therefore should be gated by mode-specific methods."),
        ("DBGCTRL", "DBGRUN", Builder, "Debug behavior is runtime-configured.")
    ],
}

#[cfg(test)]
mod tests;
