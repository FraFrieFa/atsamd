//! UART pad definitions for thumbv7em targets

use super::{AnyConfig, Capability, CharSize, Config, Duplex, Rx, Tx};
use crate::{
    gpio::AnyPin,
    sercom::*,
    typelevel::{NoneT, Sealed},
};
use core::marker::PhantomData;

//=============================================================================
// RxpoTxpo
//=============================================================================

/// Configure the `RXPO` and `TXPO` fields based on a set of [`Pads`]
///
/// According to the datasheet, the `RXPO` and `TXPO` values specify which
/// SERCOM pads are used for various functions. Moreover, depending on which
/// pads are actually in use, only certain combinations of these values make
/// sense and are valid.
///
/// This trait is implemented for valid, four-tuple combinations of
/// [`OptionalPadNum`]s. Those implementations are then lifted to the
/// corresponding [`Pads`] types.
///
/// To satisfy this trait, the combination of [`OptionalPadNum`]s must specify
/// [`PadNum`] for at least one of `RX` and `TX`. Furthermore, no
/// two [`PadNum`]s can conflict.
pub trait RxpoTxpo {
    /// `RXPO` field value
    const RXPO: u8;

    /// `RXPO` field value
    const TXPO: u8;
}

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

trait OptionalPad0 {}
impl OptionalPad0 for NoneT {}
impl OptionalPad0 for Pad0 {}

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

trait NotPad0 {}
impl NotPad0 for Pad1 {}
impl NotPad0 for Pad2 {}
impl NotPad0 for Pad3 {}

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

pub struct Empty {}
pub struct RxSimplex<RX: SomePad> {
    receive: RX,
}
pub struct TxSimplex<TX: SomePad> {
    transmit: TX,
}
pub struct HalfDuplex<IO: SomePad> {
    io: IO,
}
pub struct FullDuplex<RX: SomePad, TX: SomePad> {
    receive: RX,
    transmit: TX,
}

pub trait IoPads {
    type RX: OptionalPad;
    type TX: OptionalPad;
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
// Pads
//=============================================================================

/// Container for a set of SERCOM [`Pad`]s
///
/// See the [module-level](crate::sercom::uart) documentation for more
/// details on specifying a `Pads` type and creating instances.
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

	#[inline]
	pub fn free(self) -> (P::Pads, RTS, CTS) {
		(self.io_pads.free(), self.ready_to_send, self.clear_to_send)
	}

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

    /// Set the `RTS` [`Pad`], which is always [`Pad2`]
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

    /// Set the `CTS` [`Pad`], which is always [`Pad3`]
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
    /// Set the `RX` [`Pad`]
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

    /// Set the `TX` [`Pad`]
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

    /// Set the 'RX' ['Pad'] and `TX` [`Pad`]
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
    /// Set the `RX` [`Pad`]
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
    /// Set the `RX` [`Pad`]
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

/// Define a set of [`Pads`] using [`PinId`]s instead of [`Pin`]s
///
/// In some cases, it is more convenient to specify a set of `Pads` using
/// `PinId`s rather than `Pin`s. This alias makes it easier to do so.
///
/// The first two type parameters are the [`Sercom`] and [`IoSet`], while the
/// remaining four are effectively [`OptionalPinId`]s representing the
/// corresponding type parameters of [`Pads`], i.e. `RX`, `TX`, `RTS` & `CTS`.
/// Each of the remaining type parameters defaults to [`NoneT`].
///
/// ```
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
// not possible ->
//impl<IO : SomePad> IoMapType for (IO, IO) { type IoMapped = HalfDuplex<IO>; }
impl<RX: SomePad<PadNum: NotPad0>, TX: SomePad> IoMapType for (RX, TX) {
    type IoMapped = FullDuplex<RX, TX>;
}

pub type PadsHalfDuplexFromIds<S, I, IO, RTS = NoneT, CTS = NoneT> =
    Pads<S, I, HalfDuplex<IO>, <RTS as GetOptionalPad<S>>::Pad, <CTS as GetOptionalPad<S>>::Pad>;

//=============================================================================
// PadSet
//=============================================================================

/// Type-level function to recover the [`OptionalPad`] types from a generic set
/// of [`Pads`]
///
/// This trait is used as an interface between the [`Pads`] type and other
/// types in this module. It acts as a [type-level function], returning the
/// corresponding [`Sercom`], and [`OptionalPad`] types. It serves to
/// cut down on the total number of type parameters needed in the [`Config`]
/// struct. The [`Config`] struct doesn't need access to the [`Pad`]s directly.
/// Rather, it only needs to apply the [`SomePad`] trait bound when a `Pin` is
/// required. The [`PadSet`] trait allows each [`Config`] struct to store an
/// instance of [`Pads`] without itself being generic over all six type
/// parameters of the [`Pads`] type.
///
/// [`Pin`]: crate::gpio::Pin
/// [`Config`]: crate::sercom::uart::Config
/// [type-level function]: crate::typelevel#type-level-functions
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
// ValidPads
//=============================================================================

/// Marker trait for valid sets of [`Pads`]
///
/// This trait labels sets of [`Pads`] that satisfy the [`RxpoTxpo`]
/// trait. It guarantees to the [`Config`] struct that this set of `Pads` can
/// be configured through those traits.
///
/// [`Config`]: crate::sercom::uart::Config
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
// ValidConfig
//=============================================================================

/// Marker trait for valid UART [`Config`]urations
///
/// A functional UART peripheral must have, at a minimum either a Rx or a Tx
/// [`Pad`].
pub trait ValidConfig: AnyConfig {}

impl<P: ValidPads, C: CharSize> ValidConfig for Config<P, C> {}
