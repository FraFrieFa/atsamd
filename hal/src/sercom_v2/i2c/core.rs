//! I2C baseline for the unified SERCOM foundation.

use core::marker::PhantomData;
use atsamd_hal_macros::hal_cfg;

use crate::sercom_v2::pads::{self, IsPadSet, ReplacePad};
use crate::sercom_v2::{
    HasApbClkCtrl, IsI2cPad, IsPad, OptionalPad, Pad0, Pad1, ResourceSet, Sercom,
    SercomCoreClock, StateMarker, TakeSercom, TakeSercomCoreClock,
};
use crate::typelevel::NoneT;

const MASTER_ACT_READ: u8 = 2;
const MASTER_ACT_STOP: u8 = 3;

#[hal_cfg(any("sercom0-d11", "sercom0-d21"))]
type DataReg = u8;

#[hal_cfg("sercom0-d5x")]
type DataReg = u32;

/// Semantic I2C state markers.
pub trait I2cState: StateMarker {}

pub enum AnyRole {}
pub enum Host {}
pub enum Client {}
pub enum Smart {}
pub enum Plain {}

impl StateMarker for AnyRole {}
impl StateMarker for Host {}
impl StateMarker for Client {}
impl StateMarker for Smart {}
impl StateMarker for Plain {}

impl I2cState for AnyRole {}
impl I2cState for Host {}
impl I2cState for Client {}
impl I2cState for Smart {}
impl I2cState for Plain {}

type DefaultPads = InternalPads<pads::Pads, Roles>;
type AddRole<P, R, NP> = InternalPads<
    <<P as IsPadSet>::Pads as ReplacePad<NP>>::NewPads,
    <<P as IsPadSet>::Roles as ReplaceRole<R>>::NewRoles<NP>,
>;

pub type Pads<SDA = NoneT, SCL = NoneT> =
    AddRole<AddRole<DefaultPads, SdaRole, SDA>, SclRole, SCL>;

pub struct Roles<SDA: OptionalPad = NoneT, SCL: OptionalPad = NoneT>(PhantomData<SDA>, PhantomData<SCL>);

pub trait IsRoles {}

impl<SDA: OptionalPad, SCL: OptionalPad> IsRoles for Roles<SDA, SCL> {}

impl<SDA: OptionalPad, SCL: OptionalPad> Default for Roles<SDA, SCL> {
    fn default() -> Self {
        Self(PhantomData, PhantomData)
    }
}

pub struct SdaRole;
pub struct SclRole;

pub trait ReplaceRole<R> {
    type NewRoles<I: OptionalPad>;
    fn replace<I: OptionalPad>(self) -> Self::NewRoles<I>;
}

impl<SCL: OptionalPad> ReplaceRole<SdaRole> for Roles<NoneT, SCL> {
    type NewRoles<I: OptionalPad> = Roles<I, SCL>;
    fn replace<I: OptionalPad>(self) -> Self::NewRoles<I> { Roles(PhantomData, PhantomData) }
}
impl<SDA: OptionalPad> ReplaceRole<SclRole> for Roles<SDA, NoneT> {
    type NewRoles<I: OptionalPad> = Roles<SDA, I>;
    fn replace<I: OptionalPad>(self) -> Self::NewRoles<I> { Roles(PhantomData, PhantomData) }
}

pub struct InternalPads<
    P = pads::Pads<NoneT, NoneT, NoneT, NoneT>,
    R: IsRoles = Roles<NoneT, NoneT>,
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
    pub fn sda<I: IsPad>(self, pin: I) -> InternalPads<P::NewPads, R::NewRoles<I>>
    where
        P: ReplacePad<I>,
        R: ReplaceRole<SdaRole>,
        P::NewPads: pads::ValidPads,
        R::NewRoles<I>: IsRoles,
    {
        InternalPads {
            pads: self.pads.replace(pin),
            roles: self.roles.replace::<I>(),
        }
    }

    pub fn scl<I: IsPad>(self, pin: I) -> InternalPads<P::NewPads, R::NewRoles<I>>
    where
        P: ReplacePad<I>,
        R: ReplaceRole<SclRole>,
        P::NewPads: pads::ValidPads,
        R::NewRoles<I>: IsRoles,
    {
        InternalPads {
            pads: self.pads.replace(pin),
            roles: self.roles.replace::<I>(),
        }
    }
}

pub trait ValidPads {
    type Sercom: Sercom;
}

impl<P: pads::ValidPads, SDA: IsI2cPad<PadNum = Pad0>, SCL: IsI2cPad<PadNum = Pad1>> ValidPads
    for InternalPads<P, Roles<SDA, SCL>>
{
    type Sercom = P::Sercom;
}

/// Owned resources that justify I2C pad and clock state.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct I2cResources<Pads = InternalPads, Clock = (), Dma = (), Irqs = ()> {
    pub pads: Pads,
    pub clock: Clock,
    pub dma: Dma,
    pub irqs: Irqs,
}

impl<Pads, Clock, Dma, Irqs> ResourceSet for I2cResources<Pads, Clock, Dma, Irqs> {}

pub struct BasicI2c<S: Sercom, Pads = InternalPads, Clock = (), Dma = (), Irqs = ()> {
    sercom: S,
    resources: I2cResources<Pads, Clock, Dma, Irqs>,
    runtime: I2cRuntime,
}

#[inline]
fn i2c_master<S: Sercom>(sercom: &S) -> &crate::pac::sercom0::I2cm {
    sercom.i2cm()
}

#[inline]
fn encode_write_address(addr_7_bits: u8) -> u16 {
    (addr_7_bits as u16) << 1
}

#[inline]
fn encode_read_address(addr_7_bits: u8) -> u16 {
    ((addr_7_bits as u16) << 1) | 1
}

impl<S, Pads, Clock, Dma, Irqs>
    I2cConfig<S, crate::sercom_v2::Disabled, I2cResources<Pads, Clock, Dma, Irqs>, I2cRuntime>
where
    S: Sercom,
    Pads: ValidPads<Sercom = S>,
    Clock: SercomCoreClock<S>,
{
    pub fn enable_basic(
        self,
        mut sercom: S,
        apb: &crate::sercom_v2::ApbClkCtrl,
    ) -> BasicI2c<S, Pads, Clock, Dma, Irqs> {
        let core_clock_hz = self.resources.clock.freq_hz();
        sercom.enable_apb_clock(apb);
        let i2c = i2c_master(&sercom);

        i2c.ctrla().write(|w| w.swrst().set_bit());
        while i2c.syncbusy().read().swrst().bit_is_set() {}

        i2c.ctrla()
            .modify(|_, w| w.mode().variant(crate::pac::sercom0::i2cm::ctrla::Modeselect::I2cMaster));

        let baud_hz = self.runtime.baud.unwrap_or(100_000).max(1);
        let baud = (core_clock_hz / (2 * baud_hz)).saturating_sub(1).min(u8::MAX as u32) as u8;
        i2c.baud().modify(|_, w| unsafe { w.baud().bits(baud) });

        if let Some(runstdby) = self.runtime.runstdby {
            i2c.ctrla().modify(|_, w| w.runstdby().bit(runstdby));
        }
        if let Some(low_timeout) = self.runtime.low_timeout {
            i2c.ctrla().modify(|_, w| w.lowtouten().bit(low_timeout));
        }
        if let Some(inactive_timeout) = self.runtime.inactive_timeout {
            i2c.ctrla()
                .modify(|_, w| unsafe { w.inactout().bits(inactive_timeout) });
        }
        if let Some(hold_time) = self.runtime.hold_time {
            i2c.ctrla()
                .modify(|_, w| unsafe { w.sdahold().bits(hold_time) });
        }
        if let Some(speed) = self.runtime.speed {
            i2c.ctrla().modify(|_, w| unsafe { w.speed().bits(speed) });
        }
        if let Some(sclsm) = self.runtime.sclsm {
            i2c.ctrla().modify(|_, w| w.sclsm().bit(sclsm));
        }
        if let Some(qcmen) = self.runtime.qcmen {
            i2c.ctrlb().modify(|_, w| w.qcen().bit(qcmen));
        }
        if let Some(sexttoen) = self.runtime.sexttoen {
            i2c.ctrla().modify(|_, w| w.sexttoen().bit(sexttoen));
        }
        if let Some(mexttoen) = self.runtime.mexttoen {
            i2c.ctrla().modify(|_, w| w.mexttoen().bit(mexttoen));
        }
        if let Some(smart_mode) = self.runtime.smart_mode {
            i2c.ctrlb().modify(|_, w| w.smen().bit(smart_mode));
        }

        i2c.ctrla().modify(|_, w| w.enable().set_bit());
        while i2c.syncbusy().read().enable().bit_is_set() {}

        i2c.status().modify(|_, w| unsafe { w.busstate().bits(1) });
        while i2c.syncbusy().read().sysop().bit_is_set() {}

        BasicI2c {
            sercom,
            resources: self.resources,
            runtime: self.runtime,
        }
    }
}

impl<S, Pads, Dma, Irqs>
    I2cConfig<S, crate::sercom_v2::Disabled, I2cResources<Pads, (), Dma, Irqs>, I2cRuntime>
where
    S: Sercom,
    Pads: ValidPads<Sercom = S>,
{
    /// Materialize an ephemeral I2C config by extracting required hardware
    /// resources from a board-level resource container.
    pub fn enable_from<R>(
        self,
        resources: &mut R,
    ) -> BasicI2c<S, Pads, <R as TakeSercomCoreClock<S>>::Clock, Dma, Irqs>
    where
        R: TakeSercom<S> + TakeSercomCoreClock<S> + HasApbClkCtrl,
    {
        let sercom = resources.take_sercom();
        let clock = resources.take_sercom_core_clock();
        let apb = resources.apb_clk_ctrl();
        let config = I2cConfig::new(
            I2cResources {
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

impl<S: Sercom, Pads, Clock, Dma, Irqs> BasicI2c<S, Pads, Clock, Dma, Irqs> {
    pub fn free(self) -> (S, I2cResources<Pads, Clock, Dma, Irqs>, I2cRuntime) {
        (self.sercom, self.resources, self.runtime)
    }

    fn start_write_blocking(&mut self, address: u8) {
        let i2c = i2c_master(&self.sercom);
        i2c.addr()
            .write(|w| unsafe { w.addr().bits(encode_write_address(address)) });
        while !i2c.intflag().read().mb().bit_is_set() {}
    }

    fn start_read_blocking(&mut self, address: u8) {
        let i2c = i2c_master(&self.sercom);
        i2c.addr()
            .write(|w| unsafe { w.addr().bits(encode_read_address(address)) });
        while !i2c.intflag().read().sb().bit_is_set() {}
    }

    fn cmd_read(&mut self) {
        let i2c = i2c_master(&self.sercom);
        i2c.ctrlb().modify(|_, w| unsafe {
            w.ackact().clear_bit();
            w.cmd().bits(MASTER_ACT_READ)
        });
        while i2c.syncbusy().read().sysop().bit_is_set() {}
    }

    fn cmd_stop(&mut self) {
        let i2c = i2c_master(&self.sercom);
        i2c.ctrlb().modify(|_, w| unsafe {
            w.ackact().set_bit();
            w.cmd().bits(MASTER_ACT_STOP)
        });
        while i2c.syncbusy().read().sysop().bit_is_set() {}
    }

    pub fn write(&mut self, address: u8, bytes: &[u8]) {
        self.start_write_blocking(address);
        let i2c = i2c_master(&self.sercom);
        for byte in bytes {
            i2c.data().write(|w| unsafe { w.bits(*byte as DataReg) });
            while !i2c.intflag().read().mb().bit_is_set() {}
        }
        self.cmd_stop();
    }

    pub fn read(&mut self, address: u8, buffer: &mut [u8]) {
        if buffer.is_empty() {
            return;
        }

        self.start_read_blocking(address);
        buffer[0] = i2c_master(&self.sercom).data().read().bits() as u8;
        for dest in &mut buffer[1..] {
            self.cmd_read();
            while !i2c_master(&self.sercom).intflag().read().sb().bit_is_set() {}
            *dest = i2c_master(&self.sercom).data().read().bits() as u8;
        }
        self.cmd_stop();
    }

    pub fn write_read(&mut self, address: u8, bytes: &[u8], buffer: &mut [u8]) {
        self.start_write_blocking(address);
        let i2c = i2c_master(&self.sercom);
        for byte in bytes {
            i2c.data().write(|w| unsafe { w.bits(*byte as DataReg) });
            while !i2c.intflag().read().mb().bit_is_set() {}
        }
        self.read(address, buffer);
    }
}

crate::sercom_v2::define_sercom_protocol! {
    /// I2C protocol marker and baseline containers.
    marker: I2c,
    config: I2cConfig,
    peripheral: I2cPeripheral,
    runtime: I2cRuntime,
    builder: I2cBuilder,
    runtime_fields: {
        baud: u32,
        low_timeout: bool,
        inactive_timeout: u8,
        runstdby: bool,
        smart_mode: bool,
        hold_time: u8,
        speed: u8,
        sclsm: bool,
        qcmen: bool,
        sexttoen: bool,
        mexttoen: bool,
    },
    fields: [
        ("CTRLA", "SWRST", Function, "Lifecycle reset operation."),
        ("CTRLA", "ENABLE", Generic, "Universal enabled/disabled typestate split."),
        ("CTRLA", "MODE", Generic, "Host/client semantic role."),
        ("CTRLA", "RUNSTDBY", Builder, "Runtime standby behavior."),
        ("CTRLA", "LOWTOUTEN", Builder, "Low timeout policy."),
        ("CTRLA", "INACTOUT", Builder, "Inactive timeout value."),
        ("CTRLA", "SPEED", Builder, "Bus speed selection fits sparse runtime config."),
        ("CTRLA", "SDAHOLD", Builder, "Signal hold tuning."),
        ("CTRLA", "SCLSM", Builder, "Clock stretch policy."),
        ("CTRLA", "QCEN", Builder, "Quick command support."),
        ("CTRLA", "SEXTTOEN", Builder, "Client clock stretch timeout."),
        ("CTRLA", "MEXTTOEN", Builder, "Host clock stretch timeout."),
        ("CTRLB", "SMEN", Generic, "Smart mode changes transaction surface and ACK behavior."),
        ("CTRLB", "QCEN", Builder, "Quick command policy if present on target."),
        ("CTRLB", "CMD", Function, "Host command/write side effect path."),
        ("CTRLB", "ACKACT", Function, "Valid only in specific transfer direction states."),
        ("BAUD", "BAUD", Builder, "Sparse runtime configuration with builder API."),
        ("BAUD", "BAUDLOW", Builder, "Low period tuning."),
        ("BAUD", "HSBAUD", Builder, "High-speed host tuning."),
        ("BAUD", "HSBAUDLOW", Builder, "High-speed host low-period tuning."),
        ("ADDR", "ADDR", Function, "Transaction start/address side-effect path."),
        ("ADDR", "LEN", Builder, "Length value for DMA/smart transactions."),
        ("ADDR", "LENEN", Generic, "Length-enable changes transaction semantics."),
        ("DATA", "DATA", Function, "Volatile data path."),
        ("STATUS", "bits", Function, "Volatile status flags."),
        ("INTENSET", "bits", Function, "Interrupt enable operations are live side effects."),
        ("INTENCLR", "bits", Function, "Interrupt disable operations are live side effects."),
        ("INTFLAG", "bits", Function, "Volatile interrupt flags."),
        ("SYNCBUSY", "bits", Function, "Synchronization observation only."),
        ("DBGCTRL", "DBGRUN", Builder, "Debug behavior is runtime-configured.")
    ],
}

impl I2c {
    #[inline]
    pub fn default() -> I2cBuilder {
        I2cBuilder::new()
    }
}
