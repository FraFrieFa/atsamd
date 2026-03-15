//! # Configure the SERCOM peripherals
//!
//! The SERCOM module is used to configure the SERCOM peripherals as USART, SPI
//! or I2C interfaces.
//!
//! # Undocumented features
//!
//! The ATSAMx5x chips contain certain features that aren't documented in the
//! datasheet. These features are implemented in the HAL based on
//! experimentation with certain boards which have verifiably demonstrated that
//! those features work as intended. These undocumented features are disabled by
//! default, and can be enabled by enabling the `undoc-features` Cargo feature
//!
//! ## SAMD21:
//! * `PA00` is I2C-capable according to `circuit_playground_express`. As such,
//!   `PA00` implements [`IsI2cPad`].
//!
//! * `PA01` is I2C-capable according to `circuit_playground_express`. As such,
//!   PA01 implements [`IsI2cPad`].
//!
//! * `PB02` is I2C-capable according to `circuit_playground_express`. As such,
//!   PB02 implements [`IsI2cPad`].
//!
//! * `PB03` is I2C-capable according to `circuit_playground_express`. As such,
//!   PB03 implements [`IsI2cPad`].
//!
//! ## SAMx5x devices:
//! * `UndocIoSet1`: Implement an undocumented `IoSet` for PA16, PA17, PB22 &
//!   PB23 configured for [`Sercom1`]. The `pygamer` & `feather_m4` use this
//!   combination, but it is not listed as valid in the datasheet.
//!
//! * `UndocIoSet2`: Implement an undocumented `IoSet` for PA00, PA01, PB22 &
//!   PB23 configured for [`Sercom1`]. The `itsybitsy_m4` uses this combination,
//!   but it is not listed as valid in the datasheet.
//!
//! * [`PB02`] is I2C-capable according to `metro_m4`. As such, [`PB02`]
//!   implements [`IsI2cPad`].
//!
//! * [`PB03`] is I2C-capable according to `metro_m4`. As such, [`PB03`]
//!   implements [`IsI2cPad`].
//!
//! [`PB02`]: crate::gpio::pin::PB02
//! [`PB03`]: crate::gpio::pin::PB03
//! [`IsI2cPad`]: pad::IsI2cPad

use atsamd_hal_macros::{hal_cfg, TypeLevelTuple};
use core::marker::PhantomData;

use crate::pac;

#[hal_cfg("sercom0-d5x")]
pub type ApbClkCtrl = pac::Mclk;
#[hal_cfg(any("sercom0-d11", "sercom0-d21"))]
pub type ApbClkCtrl = pac::Pm;

pub mod pad;
pub use pad::*;

pub mod i2c;
pub mod pads;
pub mod spi;
pub mod uart;

/// Classification of a hardware field in the new SERCOM architecture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldClass {
    Generic,
    Resource,
    Builder,
    Function,
}

/// Describes how one hardware field should enter the new abstraction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FieldSpec {
    pub register: &'static str,
    pub field: &'static str,
    pub class: FieldClass,
    pub note: &'static str,
}

pub trait ProtocolDefinition {
    const NAME: &'static str;
    const FIELD_SPECS: &'static [FieldSpec];
}

pub trait StateMarker {}

impl StateMarker for () {}

pub trait SparseRuntime: Default {}

impl SparseRuntime for () {}

pub trait ResourceSet {}

impl ResourceSet for () {}

pub enum Disabled {}
pub enum Enabled {}

impl StateMarker for Disabled {}
impl StateMarker for Enabled {}

pub struct ProtocolConfig<
    S: Sercom,
    P: ProtocolDefinition,
    State: StateMarker = Disabled,
    Resources: ResourceSet = (),
    Runtime: SparseRuntime = (),
> {
    resources: Resources,
    runtime: Runtime,
    _marker: PhantomData<(S, P, State)>,
}

impl<S, P, State, Resources, Runtime> ProtocolConfig<S, P, State, Resources, Runtime>
where
    S: Sercom,
    P: ProtocolDefinition,
    State: StateMarker,
    Resources: ResourceSet,
    Runtime: SparseRuntime,
{
    pub const fn new(resources: Resources, runtime: Runtime) -> Self {
        Self {
            resources,
            runtime,
            _marker: PhantomData,
        }
    }

    pub fn resources(&self) -> &Resources {
        &self.resources
    }

    pub fn resources_mut(&mut self) -> &mut Resources {
        &mut self.resources
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut Runtime {
        &mut self.runtime
    }

    pub fn with_resources<NextResources: ResourceSet>(
        self,
        resources: NextResources,
    ) -> ProtocolConfig<S, P, State, NextResources, Runtime> {
        ProtocolConfig::new(resources, self.runtime)
    }

    pub fn with_runtime<NextRuntime: SparseRuntime>(
        self,
        runtime: NextRuntime,
    ) -> ProtocolConfig<S, P, State, Resources, NextRuntime> {
        ProtocolConfig::new(self.resources, runtime)
    }

    pub fn map_state<NextState: StateMarker>(
        self,
    ) -> ProtocolConfig<S, P, NextState, Resources, Runtime> {
        ProtocolConfig::new(self.resources, self.runtime)
    }

    pub fn enable(self) -> ProtocolPeripheral<S, P, Enabled, Resources, Runtime> {
        ProtocolPeripheral::new(self.resources, self.runtime)
    }
}

pub struct ProtocolPeripheral<
    S: Sercom,
    P: ProtocolDefinition,
    State: StateMarker = Enabled,
    Resources: ResourceSet = (),
    Runtime: SparseRuntime = (),
> {
    resources: Resources,
    runtime: Runtime,
    _marker: PhantomData<(S, P, State)>,
}

impl<S, P, State, Resources, Runtime> ProtocolPeripheral<S, P, State, Resources, Runtime>
where
    S: Sercom,
    P: ProtocolDefinition,
    State: StateMarker,
    Resources: ResourceSet,
    Runtime: SparseRuntime,
{
    pub const fn new(resources: Resources, runtime: Runtime) -> Self {
        Self {
            resources,
            runtime,
            _marker: PhantomData,
        }
    }

    pub fn resources(&self) -> &Resources {
        &self.resources
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    pub fn map_state<NextState: StateMarker>(
        self,
    ) -> ProtocolPeripheral<S, P, NextState, Resources, Runtime> {
        ProtocolPeripheral::new(self.resources, self.runtime)
    }

    pub fn disable(self) -> ProtocolConfig<S, P, Disabled, Resources, Runtime> {
        ProtocolConfig::new(self.resources, self.runtime)
    }
}

macro_rules! define_sercom_protocol {
    (
        $(#[$meta:meta])*
        marker: $marker:ident,
        config: $config:ident,
        peripheral: $peripheral:ident,
        runtime: $runtime:ident,
        builder: $builder:ident,
        runtime_fields: {
            $(
                $(#[$field_meta:meta])*
                $field:ident : $field_ty:ty
            ),* $(,)?
        },
        fields: [
            $(
                ($register:literal, $field_name:literal, $class:ident, $note:literal)
            ),* $(,)?
        ],
    ) => {
        $(#[$meta])*
        pub enum $marker {}

        impl $crate::sercom_v2::ProtocolDefinition for $marker {
            const NAME: &'static str = stringify!($marker);
            const FIELD_SPECS: &'static [$crate::sercom_v2::FieldSpec] = &[
                $(
                    $crate::sercom_v2::FieldSpec {
                        register: $register,
                        field: $field_name,
                        class: $crate::sercom_v2::FieldClass::$class,
                        note: $note,
                    }
                ),*
            ];
        }

        #[derive(Clone, Debug, Default, Eq, PartialEq)]
        pub struct $runtime {
            $(
                $(#[$field_meta])*
                pub $field: Option<$field_ty>,
            )*
        }

        impl $crate::sercom_v2::SparseRuntime for $runtime {}

        #[derive(Clone, Debug, Default, Eq, PartialEq)]
        pub struct $builder {
            runtime: $runtime,
        }

        impl $builder {
            pub fn new() -> Self {
                Self::default()
            }

            $(
                #[allow(clippy::must_use_candidate)]
                pub fn $field(mut self, value: $field_ty) -> Self {
                    self.runtime.$field = Some(value);
                    self
                }
            )*

            pub fn build(self) -> $runtime {
                self.runtime
            }
        }

        pub type $config<
            S,
            State = $crate::sercom_v2::Disabled,
            Resources = (),
            Runtime = $runtime,
        > = $crate::sercom_v2::ProtocolConfig<S, $marker, State, Resources, Runtime>;

        pub type $peripheral<
            S,
            State = $crate::sercom_v2::Enabled,
            Resources = (),
            Runtime = $runtime,
        > = $crate::sercom_v2::ProtocolPeripheral<S, $marker, State, Resources, Runtime>;
    };
}

pub(crate) use define_sercom_protocol;

pub use crate::sercom::Sercom;
pub use crate::time::Hertz;
use typenum::Unsigned;

#[hal_cfg(any("sercom0-d11", "sercom0-d21", "sercom0-d5x"))]
pub use crate::sercom::Sercom0;
#[hal_cfg(any("sercom1-d11", "sercom1-d21", "sercom1-d5x"))]
pub use crate::sercom::Sercom1;
#[hal_cfg(any("sercom2-d11", "sercom2-d21", "sercom2-d5x"))]
pub use crate::sercom::Sercom2;
#[hal_cfg(any("sercom3-d21", "sercom3-d5x"))]
pub use crate::sercom::Sercom3;
#[hal_cfg(any("sercom4-d21", "sercom4-d5x"))]
pub use crate::sercom::Sercom4;
#[hal_cfg(any("sercom5-d21", "sercom5-d5x"))]
pub use crate::sercom::Sercom5;
#[hal_cfg("sercom6-d5x")]
pub use crate::sercom::Sercom6;
#[hal_cfg("sercom7-d5x")]
pub use crate::sercom::Sercom7;

/// Type-level SERCOM ordering used by deterministic auto-selection policies.
pub trait SercomOrder: Sercom {
    type Order: Unsigned;
}

#[hal_cfg(any("sercom0-d11", "sercom0-d21", "sercom0-d5x"))]
impl SercomOrder for Sercom0 {
    type Order = typenum::U0;
}
#[hal_cfg(any("sercom1-d11", "sercom1-d21", "sercom1-d5x"))]
impl SercomOrder for Sercom1 {
    type Order = typenum::U1;
}
#[hal_cfg(any("sercom2-d11", "sercom2-d21", "sercom2-d5x"))]
impl SercomOrder for Sercom2 {
    type Order = typenum::U2;
}
#[hal_cfg(any("sercom3-d21", "sercom3-d5x"))]
impl SercomOrder for Sercom3 {
    type Order = typenum::U3;
}
#[hal_cfg(any("sercom4-d21", "sercom4-d5x"))]
impl SercomOrder for Sercom4 {
    type Order = typenum::U4;
}
#[hal_cfg(any("sercom5-d21", "sercom5-d5x"))]
impl SercomOrder for Sercom5 {
    type Order = typenum::U5;
}
#[hal_cfg("sercom6-d5x")]
impl SercomOrder for Sercom6 {
    type Order = typenum::U6;
}
#[hal_cfg("sercom7-d5x")]
impl SercomOrder for Sercom7 {
    type Order = typenum::U7;
}

/// Typed proof that the corresponding SERCOM core clock is configured.
pub trait SercomCoreClock<S: Sercom> {
    fn freq_hz(&self) -> u32;
}

/// Resource provider capable of yielding ownership of a concrete SERCOM
/// peripheral instance.
pub trait TakeSercom<S: Sercom> {
    fn take_sercom(&mut self) -> S;
}

/// Resource provider capable of yielding ownership of the corresponding SERCOM
/// core clock proof/token.
pub trait TakeSercomCoreClock<S: Sercom> {
    type Clock: SercomCoreClock<S>;
    fn take_sercom_core_clock(&mut self) -> Self::Clock;
}

/// Resource provider exposing APB clock-control registers used to enable
/// SERCOM bus clocks.
pub trait HasApbClkCtrl {
    fn apb_clk_ctrl(&self) -> &ApbClkCtrl;
}

/// PAC-backed APB access for convenience `enable_from_pac` APIs.
pub trait PacApbAccess {
    fn pac_apb(&self) -> &ApbClkCtrl;
}

/// Helper trait that extracts a SERCOM peripheral from a type-level PAC tuple.
pub trait PacTupleSercom<S: Sercom> {
    /// Tuple state after removing the SERCOM field.
    type AfterSercom;

    fn take_sercom(self) -> (S, Self::AfterSercom);
}

/// Helper trait that extracts the APB clock controller from a type-level PAC tuple.
pub trait PacTupleApb {
    /// Tuple state after removing the APB field.
    type AfterApb;

    fn take_apb(self) -> (ApbClkCtrl, Self::AfterApb);
}

pub trait IntoPeriphV2 {
    type Tuple;

    fn into_periphv2(self) -> Self::Tuple;
}

#[hal_cfg("sercom3-d5x")]
impl IntoPeriphV2 for crate::pac::Peripherals {
    type Tuple = PeriphV2Sercom3Tuple;

    fn into_periphv2(self) -> Self::Tuple {
        PeriphV2Sercom3 {
            sercom3: self.sercom3,
            mclk: self.mclk,
        }
        .into_tuple()
    }
}

#[hal_cfg("sercom3-d5x")]
#[derive(TypeLevelTuple)]
struct PeriphV2Sercom3 {
    sercom3: crate::pac::Sercom3,
    mclk: crate::pac::Mclk,
}

#[hal_cfg("sercom3-d5x")]
impl<MclkState> PacTupleSercom<Sercom3>
    for PeriphV2Sercom3Tuple<crate::typelevel_tuple::Present, MclkState>
{
    type AfterSercom = PeriphV2Sercom3Tuple<crate::typelevel_tuple::Absent, MclkState>;

    fn take_sercom(self) -> (Sercom3, Self::AfterSercom) {
        self.take_sercom3()
    }
}

#[hal_cfg("sercom3-d5x")]
impl<SercomState> PacTupleApb
    for PeriphV2Sercom3Tuple<SercomState, crate::typelevel_tuple::Present>
{
    type AfterApb = PeriphV2Sercom3Tuple<SercomState, crate::typelevel_tuple::Absent>;

    fn take_apb(self) -> (ApbClkCtrl, Self::AfterApb) {
        self.take_mclk()
    }
}

#[hal_cfg("clock-d5x")]
impl PacApbAccess for crate::pac::Peripherals {
    #[inline]
    fn pac_apb(&self) -> &ApbClkCtrl {
        &self.mclk
    }
}

#[hal_cfg(any("clock-d11", "clock-d21"))]
impl PacApbAccess for crate::pac::Peripherals {
    #[inline]
    fn pac_apb(&self) -> &ApbClkCtrl {
        &self.pm
    }
}

/// PAC-backed SERCOM acquisition for convenience `enable_from_pac` APIs.
///
/// This intentionally uses PAC `steal()` internally to avoid requiring manual
/// extraction of `sercomX` fields at the callsite.
pub trait PacTakeSercom<S: Sercom> {
    fn pac_take_sercom(&self) -> S;
}

/// Generic helper that packages the concrete hardware objects required by
/// `enable_from` without forcing board crates to define ad-hoc resource types.
pub struct EnableResources<'a, S: Sercom, C> {
    sercom: Option<S>,
    core_clock: Option<C>,
    apb: &'a ApbClkCtrl,
}

impl<'a, S: Sercom, C> EnableResources<'a, S, C> {
    #[inline]
    pub fn new(sercom: S, core_clock: C, apb: &'a ApbClkCtrl) -> Self {
        Self {
            sercom: Some(sercom),
            core_clock: Some(core_clock),
            apb,
        }
    }
}

impl<S: Sercom, C> TakeSercom<S> for EnableResources<'_, S, C> {
    #[inline]
    fn take_sercom(&mut self) -> S {
        self.sercom.take().expect("SERCOM already taken")
    }
}

impl<S: Sercom, C> TakeSercomCoreClock<S> for EnableResources<'_, S, C>
where
    C: SercomCoreClock<S>,
{
    type Clock = C;

    #[inline]
    fn take_sercom_core_clock(&mut self) -> Self::Clock {
        self.core_clock
            .take()
            .expect("SERCOM core clock already taken")
    }
}

impl<S: Sercom, C> HasApbClkCtrl for EnableResources<'_, S, C> {
    #[inline]
    fn apb_clk_ctrl(&self) -> &ApbClkCtrl {
        self.apb
    }
}

#[hal_cfg(any("sercom0-d11", "sercom0-d21", "sercom0-d5x"))]
impl PacTakeSercom<Sercom0> for crate::pac::Peripherals {
    #[inline]
    fn pac_take_sercom(&self) -> Sercom0 {
        // SAFETY: This API is a convenience escape hatch that intentionally
        // delegates singleton discipline to the caller.
        unsafe { crate::pac::Sercom0::steal() }
    }
}
#[hal_cfg(any("sercom1-d11", "sercom1-d21", "sercom1-d5x"))]
impl PacTakeSercom<Sercom1> for crate::pac::Peripherals {
    #[inline]
    fn pac_take_sercom(&self) -> Sercom1 {
        unsafe { crate::pac::Sercom1::steal() }
    }
}
#[hal_cfg(any("sercom2-d11", "sercom2-d21", "sercom2-d5x"))]
impl PacTakeSercom<Sercom2> for crate::pac::Peripherals {
    #[inline]
    fn pac_take_sercom(&self) -> Sercom2 {
        unsafe { crate::pac::Sercom2::steal() }
    }
}
#[hal_cfg(any("sercom3-d21", "sercom3-d5x"))]
impl PacTakeSercom<Sercom3> for crate::pac::Peripherals {
    #[inline]
    fn pac_take_sercom(&self) -> Sercom3 {
        unsafe { crate::pac::Sercom3::steal() }
    }
}
#[hal_cfg(any("sercom4-d21", "sercom4-d5x"))]
impl PacTakeSercom<Sercom4> for crate::pac::Peripherals {
    #[inline]
    fn pac_take_sercom(&self) -> Sercom4 {
        unsafe { crate::pac::Sercom4::steal() }
    }
}
#[hal_cfg(any("sercom5-d21", "sercom5-d5x"))]
impl PacTakeSercom<Sercom5> for crate::pac::Peripherals {
    #[inline]
    fn pac_take_sercom(&self) -> Sercom5 {
        unsafe { crate::pac::Sercom5::steal() }
    }
}
#[hal_cfg("sercom6-d5x")]
impl PacTakeSercom<Sercom6> for crate::pac::Peripherals {
    #[inline]
    fn pac_take_sercom(&self) -> Sercom6 {
        unsafe { crate::pac::Sercom6::steal() }
    }
}
#[hal_cfg("sercom7-d5x")]
impl PacTakeSercom<Sercom7> for crate::pac::Peripherals {
    #[inline]
    fn pac_take_sercom(&self) -> Sercom7 {
        unsafe { crate::pac::Sercom7::steal() }
    }
}

/// D5x helper that owns all SERCOM peripherals and provides automatic
/// extraction based on the resolved SERCOM type parameter.
#[hal_cfg("clock-d5x")]
pub struct D5xSercomResources<'a, Clock = u32> {
    pub apb: &'a ApbClkCtrl,
    pub core_clock: Clock,
    pub sercom0: Option<crate::pac::Sercom0>,
    pub sercom1: Option<crate::pac::Sercom1>,
    pub sercom2: Option<crate::pac::Sercom2>,
    pub sercom3: Option<crate::pac::Sercom3>,
    pub sercom4: Option<crate::pac::Sercom4>,
    pub sercom5: Option<crate::pac::Sercom5>,
}

#[hal_cfg("clock-d5x")]
impl<Clock> HasApbClkCtrl for D5xSercomResources<'_, Clock> {
    #[inline]
    fn apb_clk_ctrl(&self) -> &ApbClkCtrl {
        self.apb
    }
}

#[hal_cfg("clock-d5x")]
impl<S: Sercom, Clock> TakeSercomCoreClock<S> for D5xSercomResources<'_, Clock>
where
    Clock: SercomCoreClock<S> + Copy,
{
    type Clock = Clock;

    #[inline]
    fn take_sercom_core_clock(&mut self) -> Self::Clock {
        self.core_clock
    }
}

#[hal_cfg("sercom0-d5x")]
impl<Clock> TakeSercom<Sercom0> for D5xSercomResources<'_, Clock> {
    #[inline]
    fn take_sercom(&mut self) -> Sercom0 {
        self.sercom0.take().expect("SERCOM0 already taken")
    }
}
#[hal_cfg("sercom1-d5x")]
impl<Clock> TakeSercom<Sercom1> for D5xSercomResources<'_, Clock> {
    #[inline]
    fn take_sercom(&mut self) -> Sercom1 {
        self.sercom1.take().expect("SERCOM1 already taken")
    }
}
#[hal_cfg("sercom2-d5x")]
impl<Clock> TakeSercom<Sercom2> for D5xSercomResources<'_, Clock> {
    #[inline]
    fn take_sercom(&mut self) -> Sercom2 {
        self.sercom2.take().expect("SERCOM2 already taken")
    }
}
#[hal_cfg("sercom3-d5x")]
impl<Clock> TakeSercom<Sercom3> for D5xSercomResources<'_, Clock> {
    #[inline]
    fn take_sercom(&mut self) -> Sercom3 {
        self.sercom3.take().expect("SERCOM3 already taken")
    }
}
#[hal_cfg("sercom4-d5x")]
impl<Clock> TakeSercom<Sercom4> for D5xSercomResources<'_, Clock> {
    #[inline]
    fn take_sercom(&mut self) -> Sercom4 {
        self.sercom4.take().expect("SERCOM4 already taken")
    }
}
#[hal_cfg("sercom5-d5x")]
impl<Clock> TakeSercom<Sercom5> for D5xSercomResources<'_, Clock> {
    #[inline]
    fn take_sercom(&mut self) -> Sercom5 {
        self.sercom5.take().expect("SERCOM5 already taken")
    }
}

impl<S: Sercom> SercomCoreClock<S> for u32 {
    #[inline]
    fn freq_hz(&self) -> u32 {
        *self
    }
}

impl<S: Sercom> SercomCoreClock<S> for Hertz {
    #[inline]
    fn freq_hz(&self) -> u32 {
        self.to_Hz()
    }
}

#[hal_cfg(any("sercom0-d11", "sercom0-d21", "sercom0-d5x"))]
impl SercomCoreClock<Sercom0> for crate::clock::Sercom0CoreClock {
    fn freq_hz(&self) -> u32 {
        self.freq().to_Hz()
    }
}

#[hal_cfg(any("sercom1-d11", "sercom1-d21", "sercom1-d5x"))]
impl SercomCoreClock<Sercom1> for crate::clock::Sercom1CoreClock {
    fn freq_hz(&self) -> u32 {
        self.freq().to_Hz()
    }
}

#[hal_cfg(any("sercom2-d11", "sercom2-d21", "sercom2-d5x"))]
impl SercomCoreClock<Sercom2> for crate::clock::Sercom2CoreClock {
    fn freq_hz(&self) -> u32 {
        self.freq().to_Hz()
    }
}

#[hal_cfg(any("sercom3-d21", "sercom3-d5x"))]
impl SercomCoreClock<Sercom3> for crate::clock::Sercom3CoreClock {
    fn freq_hz(&self) -> u32 {
        self.freq().to_Hz()
    }
}

#[hal_cfg(any("sercom4-d21", "sercom4-d5x"))]
impl SercomCoreClock<Sercom4> for crate::clock::Sercom4CoreClock {
    fn freq_hz(&self) -> u32 {
        self.freq().to_Hz()
    }
}

#[hal_cfg(any("sercom5-d21", "sercom5-d5x"))]
impl SercomCoreClock<Sercom5> for crate::clock::Sercom5CoreClock {
    fn freq_hz(&self) -> u32 {
        self.freq().to_Hz()
    }
}

#[hal_cfg("sercom6-d5x")]
impl SercomCoreClock<Sercom6> for crate::clock::Sercom6CoreClock {
    fn freq_hz(&self) -> u32 {
        self.freq().to_Hz()
    }
}

#[hal_cfg("sercom7-d5x")]
impl SercomCoreClock<Sercom7> for crate::clock::Sercom7CoreClock {
    fn freq_hz(&self) -> u32 {
        self.freq().to_Hz()
    }
}

#[hal_cfg("clock-d5x")]
impl<S, I> SercomCoreClock<S> for crate::clock::v2::pclk::Pclk<S, I>
where
    S: Sercom + crate::clock::v2::pclk::PclkId,
    I: crate::clock::v2::pclk::PclkSourceId,
{
    fn freq_hz(&self) -> u32 {
        self.freq().to_Hz()
    }
}
