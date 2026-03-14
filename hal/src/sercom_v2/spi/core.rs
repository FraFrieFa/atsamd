//! SPI baseline for the unified SERCOM foundation.

use core::marker::PhantomData;
use atsamd_hal_macros::hal_cfg;

use crate::sercom_v2::pads::{self, IsPadSet, ReplacePad};
use crate::sercom_v2::{
    HasApbClkCtrl, IsPad, OptionalPad, Pad0, Pad1, Pad2, Pad3, ResourceSet, Sercom,
    SercomCoreClock, StateMarker, TakeSercom, TakeSercomCoreClock,
};
use crate::typelevel::NoneT;

#[hal_cfg(any("sercom0-d11", "sercom0-d21"))]
use crate::pac::sercom0::spi::ctrla::Modeselect;

#[hal_cfg("sercom0-d5x")]
use crate::pac::sercom0::spim::ctrla::Modeselect;

/// Semantic SPI state markers.
pub trait SpiState: StateMarker {}

pub enum AnyRole {}
pub enum Host {}
pub enum Client {}
pub enum RxOnly {}
pub enum TxOnly {}
pub enum Duplex {}

impl StateMarker for AnyRole {}
impl StateMarker for Host {}
impl StateMarker for Client {}
impl StateMarker for RxOnly {}
impl StateMarker for TxOnly {}
impl StateMarker for Duplex {}

impl SpiState for AnyRole {}
impl SpiState for Host {}
impl SpiState for Client {}
impl SpiState for RxOnly {}
impl SpiState for TxOnly {}
impl SpiState for Duplex {}

type DefaultPads = InternalPads<pads::Pads, Roles>;
type AddRole<P, R, NP> = InternalPads<
    <<P as IsPadSet>::Pads as ReplacePad<NP>>::NewPads,
    <<P as IsPadSet>::Roles as ReplaceRole<R>>::NewRoles<NP>,
>;

pub type Pads<DI = NoneT, DO = NoneT, CK = NoneT, SS = NoneT> =
    AddRole<AddRole<AddRole<AddRole<DefaultPads, DiRole, DI>, DoRole, DO>, CkRole, CK>, SsRole, SS>;

pub struct Roles<DI: OptionalPad = NoneT, DO: OptionalPad = NoneT, CK: OptionalPad = NoneT, SS: OptionalPad = NoneT>(
    PhantomData<DI>,
    PhantomData<DO>,
    PhantomData<CK>,
    PhantomData<SS>,
);

pub trait IsRoles {}

impl<DI: OptionalPad, DO: OptionalPad, CK: OptionalPad, SS: OptionalPad> IsRoles
    for Roles<DI, DO, CK, SS>
{
}

impl<DI: OptionalPad, DO: OptionalPad, CK: OptionalPad, SS: OptionalPad> Default for Roles<DI, DO, CK, SS> {
    fn default() -> Self {
        Self(PhantomData, PhantomData, PhantomData, PhantomData)
    }
}

pub struct DiRole;
pub struct DoRole;
pub struct CkRole;
pub struct SsRole;

pub trait ReplaceRole<R> {
    type NewRoles<I: OptionalPad>;
    fn replace<I: OptionalPad>(self) -> Self::NewRoles<I>;
}

impl<DO: OptionalPad, CK: OptionalPad, SS: OptionalPad> ReplaceRole<DiRole> for Roles<NoneT, DO, CK, SS> {
    type NewRoles<I: OptionalPad> = Roles<I, DO, CK, SS>;
    fn replace<I: OptionalPad>(self) -> Self::NewRoles<I> { Roles(PhantomData, PhantomData, PhantomData, PhantomData) }
}
impl<DI: OptionalPad, CK: OptionalPad, SS: OptionalPad> ReplaceRole<DoRole> for Roles<DI, NoneT, CK, SS> {
    type NewRoles<I: OptionalPad> = Roles<DI, I, CK, SS>;
    fn replace<I: OptionalPad>(self) -> Self::NewRoles<I> { Roles(PhantomData, PhantomData, PhantomData, PhantomData) }
}
impl<DI: OptionalPad, DO: OptionalPad, SS: OptionalPad> ReplaceRole<CkRole> for Roles<DI, DO, NoneT, SS> {
    type NewRoles<I: OptionalPad> = Roles<DI, DO, I, SS>;
    fn replace<I: OptionalPad>(self) -> Self::NewRoles<I> { Roles(PhantomData, PhantomData, PhantomData, PhantomData) }
}
impl<DI: OptionalPad, DO: OptionalPad, CK: OptionalPad> ReplaceRole<SsRole> for Roles<DI, DO, CK, NoneT> {
    type NewRoles<I: OptionalPad> = Roles<DI, DO, CK, I>;
    fn replace<I: OptionalPad>(self) -> Self::NewRoles<I> { Roles(PhantomData, PhantomData, PhantomData, PhantomData) }
}

pub struct InternalPads<
    P = pads::Pads<NoneT, NoneT, NoneT, NoneT>,
    R: IsRoles = Roles<NoneT, NoneT, NoneT, NoneT>,
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
    pub fn data_in<I: IsPad>(self, pin: I) -> InternalPads<P::NewPads, R::NewRoles<I>>
    where
        P: ReplacePad<I>,
        R: ReplaceRole<DiRole>,
        P::NewPads: pads::ValidPads,
        R::NewRoles<I>: IsRoles,
    {
        InternalPads {
            pads: self.pads.replace(pin),
            roles: self.roles.replace::<I>(),
        }
    }

    pub fn data_out<I: IsPad>(self, pin: I) -> InternalPads<P::NewPads, R::NewRoles<I>>
    where
        P: ReplacePad<I>,
        R: ReplaceRole<DoRole>,
        P::NewPads: pads::ValidPads,
        R::NewRoles<I>: IsRoles,
    {
        InternalPads {
            pads: self.pads.replace(pin),
            roles: self.roles.replace::<I>(),
        }
    }

    pub fn sclk<I: IsPad>(self, pin: I) -> InternalPads<P::NewPads, R::NewRoles<I>>
    where
        P: ReplacePad<I>,
        R: ReplaceRole<CkRole>,
        P::NewPads: pads::ValidPads,
        R::NewRoles<I>: IsRoles,
    {
        InternalPads {
            pads: self.pads.replace(pin),
            roles: self.roles.replace::<I>(),
        }
    }

    pub fn ss<I: IsPad>(self, pin: I) -> InternalPads<P::NewPads, R::NewRoles<I>>
    where
        P: ReplacePad<I>,
        R: ReplaceRole<SsRole>,
        P::NewPads: pads::ValidPads,
        R::NewRoles<I>: IsRoles,
    {
        InternalPads {
            pads: self.pads.replace(pin),
            roles: self.roles.replace::<I>(),
        }
    }
}

trait Dipo {
    const DIPO: u8;
}

crate::sercom_v2::pads::impl_const! {
    trait = Dipo;
    field = const DIPO: u8;
    NoneT => 0,
    Pad0 => 0,
    Pad1 => 1,
    Pad2 => 2,
    Pad3 => 3,
}

trait Dopo {
    const DOPO: u8;
}

#[cfg(any(feature = "samd11c", feature = "samd11d", feature = "samd21e", feature = "samd21g", feature = "samd21j", feature = "samd21gl", feature = "samd21el"))]
crate::sercom_v2::pads::impl_const! {
    trait = Dopo;
    field = const DOPO: u8;
    (Pad0, Pad1, Pad2, NoneT) => 0,
    (Pad0, Pad1, NoneT, NoneT) => 0,
    (Pad2, Pad3, Pad1, NoneT) => 0,
    (Pad2, Pad3, NoneT, NoneT) => 0,
    (Pad3, Pad1, Pad2, NoneT) => 0,
    (Pad3, Pad1, NoneT, NoneT) => 0,
    (Pad0, Pad3, Pad1, NoneT) => 0,
    (Pad0, Pad3, NoneT, NoneT) => 0,
}

#[cfg(any(feature = "samd51g", feature = "samd51j", feature = "samd51n", feature = "samd51p", feature = "same51g", feature = "same51j", feature = "same51n", feature = "same53j", feature = "same53n", feature = "same54n", feature = "same54p"))]
crate::sercom_v2::pads::impl_const! {
    trait = Dopo;
    field = const DOPO: u8;
    (NoneT, NoneT, NoneT, NoneT) => 0,
    (Pad0, Pad1, NoneT, NoneT) => 0,
    (Pad0, Pad1, Pad2, NoneT) => 0,
    (Pad3, Pad1, NoneT, NoneT) => 2,
    (Pad3, Pad1, Pad2, NoneT) => 2,
}

pub trait ValidPads {
    type Sercom: Sercom;
    type Capability: pads::CapabilityMarker;
    type SS: OptionalPad;
    const DIPO_DOPO: (u8, u8);
}

impl pads::CapabilityMarker for RxOnly {}
impl pads::CapabilityMarker for TxOnly {}
impl pads::CapabilityMarker for Duplex {}

pub trait CapabilityOf {
    type Capability: pads::CapabilityMarker;
}

impl<DI: IsPad> CapabilityOf for (DI, NoneT) {
    type Capability = RxOnly;
}

impl<DO: IsPad> CapabilityOf for (NoneT, DO) {
    type Capability = TxOnly;
}

impl<DI: IsPad, DO: IsPad> CapabilityOf for (DI, DO) {
    type Capability = Duplex;
}

impl<DI: OptionalPad, DO: OptionalPad, CK: OptionalPad, SS: OptionalPad, P: pads::ValidPads>
    ValidPads for InternalPads<P, Roles<DI, DO, CK, SS>>
where
    <DI as OptionalPad>::PadNum: Dipo,
    (<DO as OptionalPad>::PadNum, <CK as OptionalPad>::PadNum, <SS as OptionalPad>::PadNum, NoneT): Dopo,
    (DI, DO): CapabilityOf,
{
    type Sercom = P::Sercom;
    type Capability = <(DI, DO) as CapabilityOf>::Capability;
    type SS = SS;
    const DIPO_DOPO: (u8, u8) = (
        <DI as OptionalPad>::PadNum::DIPO,
        <(<DO as OptionalPad>::PadNum, <CK as OptionalPad>::PadNum, <SS as OptionalPad>::PadNum, NoneT)>::DOPO,
    );
}

/// Owned resources that justify SPI pad and clock state.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SpiResources<Pads = InternalPads, Clock = (), Dma = (), Irqs = ()> {
    pub pads: Pads,
    pub clock: Clock,
    pub dma: Dma,
    pub irqs: Irqs,
}

impl<Pads, Clock, Dma, Irqs> ResourceSet for SpiResources<Pads, Clock, Dma, Irqs> {}

pub struct BasicSpi<S: Sercom, Capability, Pads = InternalPads, Clock = (), Dma = (), Irqs = ()> {
    sercom: S,
    resources: SpiResources<Pads, Clock, Dma, Irqs>,
    runtime: SpiRuntime,
    _capability: PhantomData<Capability>,
}

pub trait CapabilityFlags {
    const RX: bool;
}

impl CapabilityFlags for RxOnly {
    const RX: bool = true;
}

impl CapabilityFlags for TxOnly {
    const RX: bool = false;
}

impl CapabilityFlags for Duplex {
    const RX: bool = true;
}

#[hal_cfg(any("sercom0-d11", "sercom0-d21"))]
#[inline]
fn spi<S: Sercom>(sercom: &S) -> &crate::pac::sercom0::Spi {
    sercom.spi()
}

#[hal_cfg("sercom0-d5x")]
#[inline]
fn spi<S: Sercom>(sercom: &S) -> &crate::pac::sercom0::Spim {
    sercom.spim()
}

impl<S, Pads, Clock, Dma, Irqs>
    SpiConfig<S, crate::sercom_v2::Disabled, SpiResources<Pads, Clock, Dma, Irqs>, SpiRuntime>
where
    S: Sercom,
    Pads: ValidPads<Sercom = S>,
    Pads::Capability: CapabilityFlags,
    Clock: SercomCoreClock<S>,
{
    pub fn enable_basic(
        self,
        mut sercom: S,
        apb: &crate::sercom_v2::ApbClkCtrl,
    ) -> BasicSpi<S, Pads::Capability, Pads, Clock, Dma, Irqs> {
        let core_clock_hz = self.resources.clock.freq_hz();
        sercom.enable_apb_clock(apb);
        let spi = spi(&sercom);

        spi.ctrla().write(|w| w.swrst().set_bit());
        while spi.syncbusy().read().swrst().bit_is_set() {}

        spi.ctrla()
            .modify(|_, w| w.mode().variant(Modeselect::SpiMaster));

        let (dipo, dopo) = Pads::DIPO_DOPO;
        spi.ctrla().modify(|_, w| unsafe {
            w.dipo().bits(dipo);
            w.dopo().bits(dopo)
        });

        if let Some(cpol) = self.runtime.cpol {
            spi.ctrla().modify(|_, w| w.cpol().bit(cpol));
        }
        if let Some(cpha) = self.runtime.cpha {
            spi.ctrla().modify(|_, w| w.cpha().bit(cpha));
        }
        if let Some(dord) = self.runtime.dord {
            spi.ctrla().modify(|_, w| w.dord().bit(dord));
        }
        if let Some(ibon) = self.runtime.ibon {
            spi.ctrla().modify(|_, w| w.ibon().bit(ibon));
        }
        if let Some(runstdby) = self.runtime.runstdby {
            spi.ctrla().modify(|_, w| w.runstdby().bit(runstdby));
        }
        if let Some(mssen) = self.runtime.mssen {
            spi.ctrlb().modify(|_, w| w.mssen().bit(mssen));
        }
        if let Some(preload_enable) = self.runtime.preload_enable {
            spi.ctrlb()
                .modify(|_, w| w.ploaden().bit(preload_enable));
        }
        if let Some(ssde) = self.runtime.ssde {
            spi.ctrlb().modify(|_, w| w.ssde().bit(ssde));
        }
        if let Some(amode) = self.runtime.amode {
            spi.ctrlb().modify(|_, w| unsafe { w.amode().bits(amode) });
        }

        #[cfg(any(feature = "samd51g", feature = "samd51j", feature = "samd51n", feature = "samd51p", feature = "same51g", feature = "same51j", feature = "same51n", feature = "same53j", feature = "same53n", feature = "same54n", feature = "same54p"))]
        {
            if let Some(data32b) = self.runtime.data32b {
                spi.ctrlc().modify(|_, w| w.data32b().bit(data32b));
            }
            if let Some(icspace) = self.runtime.icspace {
                spi.ctrlc().modify(|_, w| unsafe { w.icspace().bits(icspace) });
            }
            if let Some(length) = self.runtime.length {
                spi.length().write(|w| unsafe {
                    w.len().bits(length);
                    w.lenen().set_bit()
                });
                while spi.syncbusy().read().length().bit_is_set() {}
            }
        }

        let baud_hz = self.runtime.baud.unwrap_or(1_000_000).max(1);
        let baud_bits = (core_clock_hz / 2 / baud_hz).saturating_sub(1).min(u8::MAX as u32) as u8;
        spi.baud().write(|w| unsafe { w.baud().bits(baud_bits) });

        spi.ctrlb()
            .modify(|_, w| w.rxen().bit(<Pads::Capability as CapabilityFlags>::RX));
        while spi.syncbusy().read().ctrlb().bit_is_set() {}

        spi.ctrla().modify(|_, w| w.enable().set_bit());
        while spi.syncbusy().read().enable().bit_is_set() {}

        BasicSpi {
            sercom,
            resources: self.resources,
            runtime: self.runtime,
            _capability: PhantomData,
        }
    }
}

impl<S, Pads, Dma, Irqs>
    SpiConfig<S, crate::sercom_v2::Disabled, SpiResources<Pads, (), Dma, Irqs>, SpiRuntime>
where
    S: Sercom,
    Pads: ValidPads<Sercom = S>,
    Pads::Capability: CapabilityFlags,
{
    /// Materialize an ephemeral SPI config by extracting required hardware
    /// resources from a board-level resource container.
    pub fn enable_from<R>(
        self,
        resources: &mut R,
    ) -> BasicSpi<S, Pads::Capability, Pads, <R as TakeSercomCoreClock<S>>::Clock, Dma, Irqs>
    where
        R: TakeSercom<S> + TakeSercomCoreClock<S> + HasApbClkCtrl,
    {
        let sercom = resources.take_sercom();
        let clock = resources.take_sercom_core_clock();
        let apb = resources.apb_clk_ctrl();
        let config = SpiConfig::new(
            SpiResources {
                pads: self.resources.pads,
                clock,
                dma: self.resources.dma,
                irqs: self.resources.irqs,
            },
            self.runtime,
        );
        config.enable_basic(sercom, apb)
    }
}

impl<S: Sercom, Capability, Pads, Clock, Dma, Irqs> BasicSpi<S, Capability, Pads, Clock, Dma, Irqs> {
    pub fn free(self) -> (S, SpiResources<Pads, Clock, Dma, Irqs>, SpiRuntime) {
        (self.sercom, self.resources, self.runtime)
    }
}

impl<S: Sercom, Pads, Clock, Dma, Irqs> BasicSpi<S, TxOnly, Pads, Clock, Dma, Irqs> {
    pub fn write_u8(&mut self, byte: u8) {
        let spi = spi(&self.sercom);
        while spi.intflag().read().dre().bit_is_clear() {}
        spi.data().write(|w| unsafe { w.data().bits(byte.into()) });
    }
}

impl<S: Sercom, Pads, Clock, Dma, Irqs> BasicSpi<S, RxOnly, Pads, Clock, Dma, Irqs> {
    pub fn read_u8(&mut self) -> u8 {
        let spi = spi(&self.sercom);
        while spi.intflag().read().rxc().bit_is_clear() {}
        spi.data().read().data().bits() as u8
    }
}

impl<S: Sercom, Pads, Clock, Dma, Irqs> BasicSpi<S, Duplex, Pads, Clock, Dma, Irqs> {
    pub fn write_u8(&mut self, byte: u8) {
        let _ = self.transfer_u8(byte);
    }

    pub fn read_u8(&mut self) -> u8 {
        self.transfer_u8(0xFF)
    }

    pub fn transfer_u8(&mut self, byte: u8) -> u8 {
        let spi = spi(&self.sercom);
        while spi.intflag().read().dre().bit_is_clear() {}
        spi.data().write(|w| unsafe { w.data().bits(byte.into()) });
        while spi.intflag().read().rxc().bit_is_clear() {}
        spi.data().read().data().bits() as u8
    }
}

crate::sercom_v2::define_sercom_protocol! {
    /// SPI protocol marker and baseline containers.
    marker: Spi,
    config: SpiConfig,
    peripheral: SpiPeripheral,
    runtime: SpiRuntime,
    builder: SpiBuilder,
    runtime_fields: {
        cpol: bool,
        cpha: bool,
        dord: bool,
        baud: u32,
        ibon: bool,
        runstdby: bool,
        mssen: bool,
        preload_enable: bool,
        ssde: bool,
        amode: u8,
        data32b: bool,
        icspace: u8,
        length: u8,
    },
    fields: [
        ("CTRLA", "SWRST", Function, "Lifecycle reset operation."),
        ("CTRLA", "ENABLE", Generic, "Universal enabled/disabled typestate split."),
        ("CTRLA", "MODE", Generic, "Host/client semantic role."),
        ("CTRLA", "RUNSTDBY", Builder, "Runtime standby behavior."),
        ("CTRLA", "IBON", Builder, "Immediate overflow notification policy."),
        ("CTRLA", "DOPO", Resource, "Register-state justified by owned DO/CK/SS routing."),
        ("CTRLA", "DIPO", Resource, "Register-state justified by owned DI routing."),
        ("CTRLA", "CPOL", Builder, "Clock polarity tuning."),
        ("CTRLA", "CPHA", Builder, "Clock phase tuning."),
        ("CTRLA", "DORD", Builder, "Bit order changes encoding, not API legality."),
        ("CTRLA", "FORM", Generic, "Frame/address-recognition family affects legal behavior."),
        ("CTRLB", "RXEN", Generic, "Receive capability should be typestate."),
        ("CTRLB", "MSSEN", Resource, "Host select behavior is coupled to owned SS resources."),
        ("CTRLB", "AMODE", Builder, "Address mode is protocol configuration."),
        ("CTRLB", "SSDE", Builder, "Slave-select low detect behavior."),
        ("CTRLB", "PLOADEN", Builder, "Preload behavior is runtime-configured."),
        ("CTRLB", "CHSIZE", Generic, "Character size changes abstract word interface."),
        ("CTRLC", "DATA32B", Generic, "D5x-only extended data path mode."),
        ("CTRLC", "ICSPACE", Builder, "Sparse timing value for D5x 32-bit extension mode."),
        ("BAUD", "BAUD", Builder, "Sparse runtime configuration with builder API."),
        ("LENGTH", "LEN", Builder, "Transaction length for D5x extension mode."),
        ("LENGTH", "LENEN", Generic, "Length-enable changes transfer semantics."),
        ("ADDR", "ADDR", Function, "Address/data side-effect path, not stable typestate."),
        ("DATA", "DATA", Function, "Volatile data path."),
        ("INTENSET", "bits", Function, "Interrupt enable operations are live side effects."),
        ("INTENCLR", "bits", Function, "Interrupt disable operations are live side effects."),
        ("INTFLAG", "bits", Function, "Volatile interrupt flags."),
        ("STATUS", "bits", Function, "Volatile status flags."),
        ("SYNCBUSY", "bits", Function, "Synchronization observation only."),
        ("DBGCTRL", "DBGRUN", Builder, "Debug behavior is runtime-configured.")
    ],
}

impl Spi {
    #[inline]
    pub fn default() -> SpiBuilder {
        SpiBuilder::new()
    }
}
