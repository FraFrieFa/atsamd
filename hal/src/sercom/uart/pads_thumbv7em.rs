//! UART pad definitions for thumbv7em targets
//!
//! This module provides type-level representations of UART pad configurations
//! for SERCOM peripherals on thumbv7em targets. Using Rust’s type system,
//! it enforces correct pad combinations at compile time. The module includes:
//!
//! - Traits to compute the correct `RXPO` and `TXPO` settings from the pad configuration.
//! - Type-level mapping for various pad configurations (RX-only, TX-only, full-duplex, etc.).
//! - Methods to update pad assignments and "free" (extract) the underlying pads.

use super::{AnyConfig, Capability, CharSize, Config, Duplex, Rx, Tx};
use crate::{
    gpio::AnyPin,
    sercom::*,
    typelevel::{NoneT, Sealed},
};
use core::marker::PhantomData;

//=============================================================================
// Pad Configuration Types
//=============================================================================

/// Represents an empty pad configuration.
pub struct Empty {}

/// Represents a configuration with only an RX pad.
pub struct RxSimplex<RX: SomePad> {
    receive: RX,
}

/// Represents a configuration with only a TX pad.
pub struct TxSimplex<TX: SomePad> {
    transmit: TX,
}

/// Represents a half-duplex configuration (shared RX/TX).
pub struct HalfDuplex<IO: SomePad> {
    io: IO,
}

/// Represents a full-duplex configuration (separate RX and TX).
pub struct FullDuplex<RX: SomePad, TX: SomePad> {
    receive: RX,
    transmit: TX,
}

//=============================================================================
// IoPads Trait: Extracting Underlying Pads
//=============================================================================

/// Trait for a set of I/O pads used by the UART peripheral.
///
/// This trait encapsulates the RX and TX pad types and provides a method to
/// extract the underlying pad(s). The `free` method returns the actual hardware
/// pin(s) represented by the type-level configuration.
pub trait IoPads {
    type RX: OptionalPad;
    type TX: OptionalPad;
    /// The type of the underlying pad(s) when extracted.
    type Pads;
    fn free(self) -> Self::Pads;
}

impl IoPads for Empty {
    type RX = NoneT;
    type TX = NoneT;
    type Pads = ();
    #[inline]
    fn free(self) -> Self::Pads {
        ()
    }
}

impl<RX: SomePad> IoPads for RxSimplex<RX> {
    type RX = RX;
    type TX = NoneT;
    type Pads = RX;
    #[inline]
    fn free(self) -> Self::Pads {
        self.receive
    }
}

impl<TX: SomePad> IoPads for TxSimplex<TX> {
    type RX = NoneT;
    type TX = TX;
    type Pads = TX;
    #[inline]
    fn free(self) -> Self::Pads {
        self.transmit
    }
}

impl<IO: SomePad> IoPads for HalfDuplex<IO> {
    type RX = IO;
    type TX = IO;
    type Pads = IO;
    #[inline]
    fn free(self) -> Self::Pads {
        self.io
    }
}

impl<RX: SomePad, TX: SomePad> IoPads for FullDuplex<RX, TX> {
    type RX = RX;
    type TX = TX;
    type Pads = (RX, TX);
    #[inline]
    fn free(self) -> Self::Pads {
        (self.receive, self.transmit)
    }
}

//=============================================================================
// RxpoTxpo: Computing Pad Mappings
//=============================================================================

/// Computes the `RXPO` and `TXPO` settings based on a pad configuration.
///
/// The SERCOM peripheral uses `RXPO` and `TXPO` to select the pads for
/// data reception and transmission. This trait is implemented for valid combinations
/// of pad numbers (type-level [`OptionalPadNum`]s), ensuring:
///
/// - At least one of RX or TX is assigned a concrete pad.
/// - No conflicting pad assignments exist.
///
/// Specific pad configurations lift these implementations to the corresponding [`Pads`] types.
pub trait RxpoTxpo {
    /// The computed value for the RXPO field.
    const RXPO: u8;
    /// The computed value for the TXPO field.
    const TXPO: u8;
}

/// Helper to compute the RXPO value from a pad type (per datasheet).
trait Rxpo {
    const RXPO: u8;
}
impl Rxpo for NoneT {
    const RXPO: u8 = 0;
}
impl Rxpo for Pad0 {
    const RXPO: u8 = 0;
}
impl Rxpo for Pad1 {
    const RXPO: u8 = 1;
}
impl Rxpo for Pad2 {
    const RXPO: u8 = 2;
}
impl Rxpo for Pad3 {
    const RXPO: u8 = 3;
}

/// Marker trait indicating that a pad may serve as the primary TX pad.
trait OptionalPad0 {}
impl OptionalPad0 for NoneT {}
impl OptionalPad0 for Pad0 {}

/// Helper to compute the TXPO value from a tuple of pad types (per datasheet).
///
/// The tuple consists of:
///  - The primary TX pad number,
///  - The RTS pad number,
///  - The CTS pad number.
trait Txpo {
    const TXPO: u8;
}
impl<TX: OptionalPad0> Txpo for (TX, NoneT, NoneT) {
    const TXPO: u8 = 0;
}
impl<TX: OptionalPad0> Txpo for (TX, Pad2, Pad3) {
    const TXPO: u8 = 2;
}
impl<TX: OptionalPad0> Txpo for (TX, Pad2, NoneT) {
    const TXPO: u8 = 3;
}

/// Prevents the use of `Pad0` as an RX pad in certain configurations.
trait NotPad0 {}
impl NotPad0 for Pad1 {}
impl NotPad0 for Pad2 {}
impl NotPad0 for Pad3 {}

//=============================================================================
// Pads: High-Level Pad Container
//=============================================================================

/// Container for a set of SERCOM pads along with optional RTS/CTS signals.
///
/// This type-level container couples the pad configuration (`io_pads`) with control
/// signals (`ready_to_send` and `clear_to_send`). It is parameterized by:
/// - `S`: the SERCOM peripheral.
/// - `I`: the I/O set.
/// - `P`: the pad configuration (implements [`IoPads`]).
/// - `RTS`/`CTS`: optional control pad types.
pub struct Pads<S, I, P = Empty, RTS = NoneT, CTS = NoneT>
where
    S: Sercom,
    I: IoSet,
    P: IoPads,
    RTS: OptionalPad,
    CTS: OptionalPad,
{
    sercom: PhantomData<S>,
    ioset: PhantomData<I>,
    io_pads: P,
    ready_to_send: RTS,
    clear_to_send: CTS,
}

//=============================================================================
// RxpoTxpo Implementations for Pads
//=============================================================================

impl<S, I, RX, TX, RTS, CTS> RxpoTxpo for Pads<S, I, FullDuplex<RX, TX>, RTS, CTS>
where
    S: Sercom,
    I: IoSet,
    RX: SomePad<PadNum: Rxpo + NotPad0>,
    TX: SomePad,
    (TX::PadNum, RTS::PadNum, CTS::PadNum): Txpo,
    RTS: OptionalPad,
    CTS: OptionalPad,
{
    const RXPO: u8 = RX::PadNum::RXPO;
    const TXPO: u8 = <(TX::PadNum, RTS::PadNum, CTS::PadNum)>::TXPO;
}

impl<S, I, IO, RTS, CTS> RxpoTxpo for Pads<S, I, HalfDuplex<IO>, RTS, CTS>
where
    S: Sercom,
    I: IoSet,
    IO: SomePad<PadNum: Rxpo>,
    (IO::PadNum, RTS::PadNum, CTS::PadNum): Txpo,
    RTS: OptionalPad,
    CTS: OptionalPad,
{
    const RXPO: u8 = IO::PadNum::RXPO;
    const TXPO: u8 = <(IO::PadNum, RTS::PadNum, CTS::PadNum)>::TXPO;
}

impl<S, I, RX, RTS, CTS> RxpoTxpo for Pads<S, I, RxSimplex<RX>, RTS, CTS>
where
    S: Sercom,
    I: IoSet,
    RX: SomePad<PadNum: Rxpo>,
    (NoneT, RTS::PadNum, CTS::PadNum): Txpo,
    RTS: OptionalPad,
    CTS: OptionalPad,
{
    const RXPO: u8 = RX::PadNum::RXPO;
    const TXPO: u8 = <(NoneT, RTS::PadNum, CTS::PadNum)>::TXPO;
}

impl<S, I, TX, RTS, CTS> RxpoTxpo for Pads<S, I, TxSimplex<TX>, RTS, CTS>
where
    S: Sercom,
    I: IoSet,
    TX: SomePad,
    (TX::PadNum, RTS::PadNum, CTS::PadNum): Txpo,
    RTS: OptionalPad,
    CTS: OptionalPad,
{
    const RXPO: u8 = NoneT::RXPO;
    const TXPO: u8 = <(TX::PadNum, RTS::PadNum, CTS::PadNum)>::TXPO;
}

//=============================================================================
// Pads Builder Methods
//=============================================================================

impl<S: Sercom, I: IoSet> Default for Pads<S, I> {
    fn default() -> Self {
        Self {
            sercom: PhantomData,
            ioset: PhantomData,
            io_pads: Empty {},
            ready_to_send: NoneT,
            clear_to_send: NoneT,
        }
    }
}

impl<S, I, P, RTS, CTS> Pads<S, I, P, RTS, CTS>
where
    S: Sercom,
    I: IoSet,
    P: IoPads,
    RTS: OptionalPad,
    CTS: OptionalPad,
{
    /// Frees the configured pads, returning a tuple of (underlying pad(s), RTS, CTS).
    #[inline]
    pub fn free(self) -> (P::Pads, RTS, CTS) {
        (self.io_pads.free(), self.ready_to_send, self.clear_to_send)
    }

    /// Internal helper: updates the pad configuration while preserving SERCOM, I/O set, and control signals.
    #[inline]
    fn update_io<P2: IoPads>(self, io_pads: P2) -> Pads<S, I, P2, RTS, CTS> {
        Pads {
            sercom: self.sercom,
            ioset: self.ioset,
            io_pads,
            ready_to_send: self.ready_to_send,
            clear_to_send: self.clear_to_send,
        }
    }

    /// Sets the RTS pad (assigned to pad number 2).
    #[inline]
    pub fn rts<Id>(self, pin: impl AnyPin<Id = Id>) -> Pads<S, I, P, Pad<S, Id>, CTS>
    where
        Id: GetPad<S>,
        Pad<S, Id>: InIoSet<I>,
    {
        Pads {
            sercom: self.sercom,
            ioset: self.ioset,
            io_pads: self.io_pads,
            ready_to_send: pin.into().into_mode(),
            clear_to_send: self.clear_to_send,
        }
    }

    /// Sets the CTS pad (assigned to pad number 3).
    #[inline]
    pub fn cts<Id>(self, pin: impl AnyPin<Id = Id>) -> Pads<S, I, P, RTS, Pad<S, Id>>
    where
        Id: GetPad<S>,
        Pad<S, Id>: InIoSet<I>,
    {
        Pads {
            sercom: self.sercom,
            ioset: self.ioset,
            io_pads: self.io_pads,
            ready_to_send: self.ready_to_send,
            clear_to_send: pin.into().into_mode(),
        }
    }
}

impl<S, I, RTS, CTS> Pads<S, I, Empty, RTS, CTS>
where
    S: Sercom,
    I: IoSet,
    RTS: OptionalPad,
    CTS: OptionalPad,
{
    /// Assigns an RX pad, transitioning from an empty pad configuration.
    #[inline]
    pub fn rx<Id>(self, pin: impl AnyPin<Id = Id>) -> Pads<S, I, RxSimplex<Pad<S, Id>>, RTS, CTS>
    where
        Id: GetPad<S>,
        Pad<S, Id>: InIoSet<I>,
    {
        self.update_io(RxSimplex {
            receive: pin.into().into_mode(),
        })
    }

    /// Assigns a TX pad, transitioning from an empty pad configuration.
    #[inline]
    pub fn tx<Id>(self, pin: impl AnyPin<Id = Id>) -> Pads<S, I, TxSimplex<Pad<S, Id>>, RTS, CTS>
    where
        Id: GetPad<S>,
        Pad<S, Id>: InIoSet<I>,
    {
        self.update_io(TxSimplex {
            transmit: pin.into().into_mode(),
        })
    }

    /// Assigns a half-duplex (shared RX/TX) pad.
    #[inline]
    pub fn io<Id>(self, pin: impl AnyPin<Id = Id>) -> Pads<S, I, HalfDuplex<Pad<S, Id>>, RTS, CTS>
    where
        Id: GetPad<S>,
        Pad<S, Id>: InIoSet<I>,
    {
        self.update_io(HalfDuplex {
            io: pin.into().into_mode(),
        })
    }
}

impl<S, I, TX, RTS, CTS> Pads<S, I, TxSimplex<TX>, RTS, CTS>
where
    S: Sercom,
    I: IoSet,
    TX: SomePad,
    RTS: OptionalPad,
    CTS: OptionalPad,
{
    /// Converts a TX-only configuration to full-duplex by assigning an RX pad.
    #[inline]
    pub fn rx<Id>(
        self,
        pin: impl AnyPin<Id = Id>,
    ) -> Pads<S, I, FullDuplex<Pad<S, Id>, TX>, RTS, CTS>
    where
        Id: GetPad<S>,
        Pad<S, Id>: InIoSet<I>,
    {
        Pads {
            sercom: self.sercom,
            ioset: self.ioset,
            io_pads: FullDuplex {
                transmit: self.io_pads.transmit,
                receive: pin.into().into_mode(),
            },
            ready_to_send: self.ready_to_send,
            clear_to_send: self.clear_to_send,
        }
    }
}

impl<S, I, RX, RTS, CTS> Pads<S, I, RxSimplex<RX>, RTS, CTS>
where
    S: Sercom,
    I: IoSet,
    RX: SomePad,
    RTS: OptionalPad,
    CTS: OptionalPad,
{
    /// Converts an RX-only configuration to full-duplex by assigning a TX pad.
    #[inline]
    pub fn tx<Id>(
        self,
        pin: impl AnyPin<Id = Id>,
    ) -> Pads<S, I, FullDuplex<RX, Pad<S, Id>>, RTS, CTS>
    where
        Id: GetPad<S>,
        Pad<S, Id>: InIoSet<I>,
    {
        Pads {
            sercom: self.sercom,
            ioset: self.ioset,
            io_pads: FullDuplex {
                transmit: pin.into().into_mode(),
                receive: self.io_pads.receive,
            },
            ready_to_send: self.ready_to_send,
            clear_to_send: self.clear_to_send,
        }
    }
}

/// Type alias to create [`Pads`] using pin identifiers instead of concrete pins.
///
/// The first two parameters are the [`Sercom`] and [`IoSet`]; the remaining four
/// are the corresponding RX, TX, RTS, and CTS pin identifiers (defaulting to [`NoneT`]).
///
/// # Example
///
/// ```rust
/// use atsamd_hal::pac::Peripherals;
/// use atsamd_hal::gpio::{PA08, PA09, Pins};
/// use atsamd_hal::sercom::{Sercom0, uart};
/// use atsamd_hal::sercom::pad::IoSet1;
/// use atsamd_hal::typelevel::NoneT;
///
/// pub type Pads = uart::PadsFromIds<Sercom0, IoSet1, PA09, PA08>;
///
/// pub fn create_pads() -> Pads {
///     let peripherals = Peripherals::take().unwrap();
///     let pins = Pins::new(peripherals.PORT);
///     uart::Pads::default().rx(pins.pa09).tx(pins.pa08)
/// }
/// ```
///
/// [`Pin`]: crate::gpio::Pin
/// [`PinId`]: crate::gpio::PinId
/// [`OptionalPinId`]: crate::gpio::OptionalPinId
pub type PadsFromIds<S, I, RX = NoneT, TX = NoneT, RTS = NoneT, CTS = NoneT> = Pads<
    S,
    I,
    <(RX, TX) as IoMapType>::IoMapped,
    <RTS as GetOptionalPad<S>>::Pad,
    <CTS as GetOptionalPad<S>>::Pad,
>;

/// Maps a tuple of pin identifiers to a type-level pad configuration.
pub trait IoMapType {
    type IoMapped;
}
impl IoMapType for (NoneT, NoneT) {
    type IoMapped = Empty;
}
impl<RX: SomePad> IoMapType for (RX, NoneT) {
    type IoMapped = RxSimplex<RX>;
}
impl<TX: SomePad> IoMapType for (NoneT, TX) {
    type IoMapped = TxSimplex<TX>;
}
// Half-duplex (IO, IO) is not allowed.
impl<RX: SomePad<PadNum: NotPad0>, TX: SomePad> IoMapType for (RX, TX) {
    type IoMapped = FullDuplex<RX, TX>;
}

/// Type alias for half-duplex pad configurations using pin identifiers.
pub type PadsHalfDuplexFromIds<S, I, IO, RTS = NoneT, CTS = NoneT> =
    Pads<S, I, HalfDuplex<IO>, <RTS as GetOptionalPad<S>>::Pad, <CTS as GetOptionalPad<S>>::Pad>;

//=============================================================================
// PadSet: Extracting Type-Level Pad Information
//=============================================================================

/// Type-level function to extract pad types from a [`Pads`] configuration.
///
/// This trait bridges the [`Pads`] type with other types in the module by returning
/// the associated SERCOM, I/O set, and individual RX, TX, RTS, and CTS pad types. This
/// reduces the number of type parameters required in the [`Config`] struct.
///
/// [`Config`]: crate::sercom::uart::Config
pub trait PadSet: Sealed {
    type Sercom: Sercom;
    type IoSet: IoSet;
    type Rx: OptionalPad;
    type Tx: OptionalPad;
    type Rts: OptionalPad;
    type Cts: OptionalPad;
}

impl<S, I, P, RTS, CTS> Sealed for Pads<S, I, P, RTS, CTS>
where
    S: Sercom,
    I: IoSet,
    P: IoPads,
    RTS: OptionalPad,
    CTS: OptionalPad,
{
}
impl<S, I, P, RTS, CTS> PadSet for Pads<S, I, P, RTS, CTS>
where
    S: Sercom,
    I: IoSet,
    P: IoPads,
    RTS: OptionalPad,
    CTS: OptionalPad,
{
    type Sercom = S;
    type IoSet = I;
    type Rx = P::RX;
    type Tx = P::TX;
    type Rts = RTS;
    type Cts = CTS;
}

//=============================================================================
// ValidPads: Ensuring a Valid Pad Configuration
//=============================================================================

/// Marker trait for valid pad configurations.
///
/// A type implementing this trait guarantees that its pad configuration satisfies
/// the constraints of [`RxpoTxpo`] and can be used to configure a UART peripheral.
/// The associated `Capability` indicates if the configuration supports RX-only,
/// TX-only, or full-duplex communication.
pub trait ValidPads: PadSet + RxpoTxpo {
    type Capability: Capability;
}

impl<S, I, RX, RTS> ValidPads for Pads<S, I, RxSimplex<RX>, RTS, NoneT>
where
    S: Sercom,
    I: IoSet,
    RX: SomePad,
    RTS: OptionalPad,
    Self: PadSet + RxpoTxpo,
{
    type Capability = Rx;
}

impl<S, I, TX, CTS> ValidPads for Pads<S, I, TxSimplex<TX>, NoneT, CTS>
where
    S: Sercom,
    I: IoSet,
    TX: SomePad,
    CTS: OptionalPad,
    Self: PadSet + RxpoTxpo,
{
    type Capability = Tx;
}

impl<S, I, RX, TX, RTS, CTS> ValidPads for Pads<S, I, FullDuplex<RX, TX>, RTS, CTS>
where
    S: Sercom,
    I: IoSet,
    RX: SomePad,
    TX: SomePad,
    RTS: OptionalPad,
    CTS: OptionalPad,
    Self: PadSet + RxpoTxpo,
{
    type Capability = Duplex;
}

impl<S, I, IO, RTS, CTS> ValidPads for Pads<S, I, HalfDuplex<IO>, RTS, CTS>
where
    S: Sercom,
    I: IoSet,
    IO: SomePad,
    RTS: OptionalPad,
    CTS: OptionalPad,
    Self: PadSet + RxpoTxpo,
{
    type Capability = Duplex;
}

//=============================================================================
// ValidConfig: Minimum Requirements for a Functional UART
//=============================================================================

/// Marker trait for valid UART configurations.
///
/// A UART peripheral must have at least one of RX or TX pads configured.
/// This trait restricts [`Config`] to accept only configurations that meet this requirement.
pub trait ValidConfig: AnyConfig {}
impl<P: ValidPads, C: CharSize> ValidConfig for Config<P, C> {}
