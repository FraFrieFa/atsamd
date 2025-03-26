use crate::gpio::*;
use crate::sercom::*;
use crate::typelevel::NoneT;
use core::marker::PhantomData;

trait OptionalPadNum {}
impl<P: PadNum> OptionalPadNum for P {}
impl OptionalPadNum for NoneT {}

/// Bundles the pad role type parameters (rx, tx, clk, rts, cts).
pub struct PadRoles<
    Rx: OptionalPadNum = NoneT,
    Tx: OptionalPadNum = NoneT,
    Clk: OptionalPadNum = NoneT,
    Rts: OptionalPadNum = NoneT,
    Cts: OptionalPadNum = NoneT,
> {
    rx: PhantomData<Rx>,
    tx: PhantomData<Tx>,
    clk: PhantomData<Clk>,
    rts: PhantomData<Rts>,
    cts: PhantomData<Cts>,
}

impl Default for PadRoles<NoneT, NoneT, NoneT, NoneT, NoneT> {
    fn default() -> Self {
        PadRoles {
            rx: PhantomData,
            tx: PhantomData,
            clk: PhantomData,
            rts: PhantomData,
            cts: PhantomData,
        }
    }
}

/// The main Pads struct now bundles the five pad role generics into one `R` parameter.
pub struct Pads<
    S: Sercom,
    P0: OptionalPin = NoneT,
    P1: OptionalPin = NoneT,
    P2: OptionalPin = NoneT,
    P3: OptionalPin = NoneT,
    R: PadRolesInterface = PadRoles<NoneT, NoneT, NoneT, NoneT, NoneT>,
> {
    s: S,
    p0: P0,
    p1: P1,
    p2: P2,
    p3: P3,
    roles: R,
}

impl<S: Sercom> Pads<S> {
    /// Constructs a default Pads instance with no physical pins and default pad roles.
    fn default(s: S) -> Self {
        Pads {
            s,
            p0: NoneT,
            p1: NoneT,
            p2: NoneT,
            p3: NoneT,
            roles: PadRoles::default(),
        }
    }
}

pub trait PadRolesInterface {
    type Rx: OptionalPadNum;
    type Tx: OptionalPadNum;
    type Clk: OptionalPadNum;
    type Rts: OptionalPadNum;
    type Cts: OptionalPadNum;

    fn access(self) -> PadRoles<Self::Rx, Self::Tx, Self::Clk, Self::Rts, Self::Cts>;
}

impl<
        Rx: OptionalPadNum,
        Tx: OptionalPadNum,
        Clk: OptionalPadNum,
        Rts: OptionalPadNum,
        Cts: OptionalPadNum,
    > PadRolesInterface for PadRoles<Rx, Tx, Clk, Rts, Cts>
{
    type Rx = Rx;
    type Tx = Tx;
    type Clk = Clk;
    type Rts = Rts;
    type Cts = Cts;
    #[inline]
    fn access(self) -> PadRoles<Rx, Tx, Clk, Rts, Cts> {
        self
    }
}

/// Interface to access the underlying Pads with its complete type state.
pub trait PadsInterface {
    type SercomType: Sercom;
    type P0: OptionalPin;
    type P1: OptionalPin;
    type P2: OptionalPin;
    type P3: OptionalPin;
    type Roles: PadRolesInterface;

    fn access(self) -> Pads<Self::SercomType, Self::P0, Self::P1, Self::P2, Self::P3, Self::Roles>;
}

impl<
        S: Sercom,
        P0: OptionalPin,
        P1: OptionalPin,
        P2: OptionalPin,
        P3: OptionalPin,
        R: PadRolesInterface,
    > PadsInterface for Pads<S, P0, P1, P2, P3, R>
{
    type SercomType = S;
    type P0 = P0;
    type P1 = P1;
    type P2 = P2;
    type P3 = P3;
    type Roles = R;
    #[inline]
    fn access(self) -> Pads<S, P0, P1, P2, P3, Self::Roles> {
        self
    }
}

/// Trait for replacing a physical pad (P0–P3) with a new pin.
trait ReplacePad<P: PadNum, NewPin: SomePin> {
    type Output: PadsInterface;
    fn replace_pad(self, new_pin: NewPin) -> Self::Output;
}

// Replace for Pad0.
impl<
        S: Sercom,
        P0: SomePin,
        P1: OptionalPin,
        P2: OptionalPin,
        P3: OptionalPin,
        R: PadRolesInterface,
    > ReplacePad<Pad0, P0> for Pads<S, NoneT, P1, P2, P3, R>
{
    type Output = Pads<S, P0, P1, P2, P3, R>;
    fn replace_pad(self, new_pin: P0) -> Self::Output {
        Pads {
            s: self.s,
            p0: new_pin,
            p1: self.p1,
            p2: self.p2,
            p3: self.p3,
            roles: self.roles,
        }
    }
}

// Replace for Pad1.
impl<
        S: Sercom,
        P0: OptionalPin,
        P1: SomePin,
        P2: OptionalPin,
        P3: OptionalPin,
        R: PadRolesInterface,
    > ReplacePad<Pad1, P1> for Pads<S, P0, NoneT, P2, P3, R>
{
    type Output = Pads<S, P0, P1, P2, P3, R>;
    fn replace_pad(self, new_pin: P1) -> Self::Output {
        Pads {
            s: self.s,
            p0: self.p0,
            p1: new_pin,
            p2: self.p2,
            p3: self.p3,
            roles: self.roles,
        }
    }
}

// Replace for Pad2.
impl<
        S: Sercom,
        P0: OptionalPin,
        P1: OptionalPin,
        P2: SomePin,
        P3: OptionalPin,
        R: PadRolesInterface,
    > ReplacePad<Pad2, P2> for Pads<S, P0, P1, NoneT, P3, R>
{
    type Output = Pads<S, P0, P1, P2, P3, R>;
    fn replace_pad(self, new_pin: P2) -> Self::Output {
        Pads {
            s: self.s,
            p0: self.p0,
            p1: self.p1,
            p2: new_pin,
            p3: self.p3,
            roles: self.roles,
        }
    }
}

// Replace for Pad3.
impl<
        S: Sercom,
        P0: OptionalPin,
        P1: OptionalPin,
        P2: OptionalPin,
        P3: SomePin,
        R: PadRolesInterface,
    > ReplacePad<Pad3, P3> for Pads<S, P0, P1, P2, NoneT, R>
{
    type Output = Pads<S, P0, P1, P2, P3, R>;
    fn replace_pad(self, new_pin: P3) -> Self::Output {
        Pads {
            s: self.s,
            p0: self.p0,
            p1: self.p1,
            p2: self.p2,
            p3: new_pin,
            roles: self.roles,
        }
    }
}

impl<
        S: Sercom,
        P0: OptionalPin,
        P1: OptionalPin,
        P2: OptionalPin,
        P3: OptionalPin,
        R: PadRolesInterface,
    > Pads<S, P0, P1, P2, P3, R>
{
    /// A generic helper to replace one of the physical pads.
    fn replace_pad<P>(
        self,
        new_pin: P,
    ) -> <Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output
    where
        P: SomePin,
        P::Id: GetPad<S>,
        Self: ReplacePad<<P::Id as GetPad<S>>::PadNum, P>,
    {
        <Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::replace_pad(self, new_pin)
    }
}

/// Implement the `rx` method by first replacing the physical pad using `replace_pad`
/// and then updating the rx role in the PadRoles bundle.
impl<
        S: Sercom,
        P0: OptionalPin,
        P1: OptionalPin,
        P2: OptionalPin,
        P3: OptionalPin,
        Tx: OptionalPadNum,
        Clk: OptionalPadNum,
        Rts: OptionalPadNum,
        Cts: OptionalPadNum,
    > Pads<S, P0, P1, P2, P3, PadRoles<NoneT, Tx, Clk, Rts, Cts>>
{
    fn rx<P: SomePin>(
        self,
        new_pin: P,
    ) -> Pads<
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::SercomType,
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::P0,
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::P1,
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::P2,
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::P3,
    	PadRoles<
    		<P::Id as GetPad<S>>::PadNum,
    		<<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::Roles as PadRolesInterface>::Tx,
    		<<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::Roles as PadRolesInterface>::Clk,
    		<<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::Roles as PadRolesInterface>::Rts,
    		<<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::Roles as PadRolesInterface>::Cts
		>
	>
    where
        P::Id: GetPad<S>,
        Self: ReplacePad<<P::Id as GetPad<S>>::PadNum, P>,
    {

        let replaced = self.replace_pad(new_pin).access();
        let roles = replaced.roles.access();
        Pads {
            s: replaced.s,
            p0: replaced.p0,
            p1: replaced.p1,
            p2: replaced.p2,
            p3: replaced.p3,
            roles: PadRoles {
                rx: PhantomData,
                tx: roles.tx,
                clk: roles.clk,
                rts: roles.rts,
                cts: roles.cts,
            },
        }
    }
}



impl<
        S: Sercom,
        P0: OptionalPin,
        P1: OptionalPin,
        P2: OptionalPin,
        P3: OptionalPin,
        Rx: OptionalPadNum,
        Clk: OptionalPadNum,
        Rts: OptionalPadNum,
        Cts: OptionalPadNum,
    > Pads<S, P0, P1, P2, P3, PadRoles<Rx, NoneT, Clk, Rts, Cts>>
{
    fn tx<P: SomePin>(
        self,
        new_pin: P,
    ) -> Pads<
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::SercomType,
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::P0,
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::P1,
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::P2,
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::P3,
    	PadRoles<
    		<<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::Roles as PadRolesInterface>::Rx,
    		<P::Id as GetPad<S>>::PadNum,
    		<<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::Roles as PadRolesInterface>::Clk,
    		<<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::Roles as PadRolesInterface>::Rts,
    		<<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::Roles as PadRolesInterface>::Cts
		>
	>
    where
        P::Id: GetPad<S>,
        Self: ReplacePad<<P::Id as GetPad<S>>::PadNum, P>,
    {

        let replaced = self.replace_pad(new_pin).access();
        let roles = replaced.roles.access();
        Pads {
            s: replaced.s,
            p0: replaced.p0,
            p1: replaced.p1,
            p2: replaced.p2,
            p3: replaced.p3,
            roles: PadRoles {
                rx: roles.rx,
                tx: PhantomData,
                clk: roles.clk,
                rts: roles.rts,
                cts: roles.cts,
            },
        }
    }
}

impl<
        S: Sercom,
        P0: OptionalPin,
        P1: OptionalPin,
        P2: OptionalPin,
        P3: OptionalPin,
        Clk: OptionalPadNum,
        Rts: OptionalPadNum,
        Cts: OptionalPadNum,
    > Pads<S, P0, P1, P2, P3, PadRoles<NoneT, NoneT, Clk, Rts, Cts>>
{
    fn io<P: SomePin>(
        self,
        new_pin: P,
    ) -> Pads<
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::SercomType,
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::P0,
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::P1,
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::P2,
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::P3,
    	PadRoles<
    		<P::Id as GetPad<S>>::PadNum,
    		<P::Id as GetPad<S>>::PadNum,
    		<<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::Roles as PadRolesInterface>::Clk,
    		<<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::Roles as PadRolesInterface>::Rts,
    		<<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::Roles as PadRolesInterface>::Cts
		>
	>
    where
        P::Id: GetPad<S>,
        Self: ReplacePad<<P::Id as GetPad<S>>::PadNum, P>,
    {

        let replaced = self.replace_pad(new_pin).access();
        let roles = replaced.roles.access();
        Pads {
            s: replaced.s,
            p0: replaced.p0,
            p1: replaced.p1,
            p2: replaced.p2,
            p3: replaced.p3,
            roles: PadRoles {
                rx: PhantomData,
                tx: PhantomData,
                clk: roles.clk,
                rts: roles.rts,
                cts: roles.cts,
            },
        }
    }
}


impl<
        S: Sercom,
        P0: OptionalPin,
        P1: OptionalPin,
        P2: OptionalPin,
        P3: OptionalPin,
        Rx: OptionalPadNum,
        Tx: OptionalPadNum,
        Rts: OptionalPadNum,
        Cts: OptionalPadNum,
    > Pads<S, P0, P1, P2, P3, PadRoles<Rx, Tx, NoneT, Rts, Cts>>
{
    fn clk<P: SomePin>(
        self,
        new_pin: P,
    ) -> Pads<
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::SercomType,
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::P0,
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::P1,
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::P2,
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::P3,
    	PadRoles<
    		<<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::Roles as PadRolesInterface>::Rx,
    		<<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::Roles as PadRolesInterface>::Tx,
    		<P::Id as GetPad<S>>::PadNum,
    		<<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::Roles as PadRolesInterface>::Rts,
    		<<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::Roles as PadRolesInterface>::Cts
		>
	>
    where
        P::Id: GetPad<S>,
        Self: ReplacePad<<P::Id as GetPad<S>>::PadNum, P>,
    {

        let replaced = self.replace_pad(new_pin).access();
        let roles = replaced.roles.access();
        Pads {
            s: replaced.s,
            p0: replaced.p0,
            p1: replaced.p1,
            p2: replaced.p2,
            p3: replaced.p3,
            roles: PadRoles {
                rx: roles.rx,
                tx: roles.tx,
                clk: PhantomData,
                rts: roles.rts,
                cts: roles.cts,
            },
        }
    }
}


impl<
        S: Sercom,
        P0: OptionalPin,
        P1: OptionalPin,
        P2: OptionalPin,
        P3: OptionalPin,
        Rx: OptionalPadNum,
        Tx: OptionalPadNum,
        Clk: OptionalPadNum,
        Cts: OptionalPadNum,
    > Pads<S, P0, P1, P2, P3, PadRoles<Rx, Tx, Clk, NoneT, Cts>>
{
    fn rts<P: SomePin>(
        self,
        new_pin: P,
    ) -> Pads<
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::SercomType,
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::P0,
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::P1,
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::P2,
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::P3,
    	PadRoles<
    		<<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::Roles as PadRolesInterface>::Rx,
    		<<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::Roles as PadRolesInterface>::Tx,
    		<<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::Roles as PadRolesInterface>::Clk,
    		<P::Id as GetPad<S>>::PadNum,
    		<<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::Roles as PadRolesInterface>::Cts
		>
	>
    where
        P::Id: GetPad<S>,
        Self: ReplacePad<<P::Id as GetPad<S>>::PadNum, P>,
    {

        let replaced = self.replace_pad(new_pin).access();
        let roles = replaced.roles.access();
        Pads {
            s: replaced.s,
            p0: replaced.p0,
            p1: replaced.p1,
            p2: replaced.p2,
            p3: replaced.p3,
            roles: PadRoles {
                rx: roles.rx,
                tx: roles.tx,
                clk: roles.clk,
                rts: PhantomData,
                cts: roles.cts,
            },
        }
    }
}


impl<
        S: Sercom,
        P0: OptionalPin,
        P1: OptionalPin,
        P2: OptionalPin,
        P3: OptionalPin,
        Rx: OptionalPadNum,
        Tx: OptionalPadNum,
        Clk: OptionalPadNum,
        Rts: OptionalPadNum,
    > Pads<S, P0, P1, P2, P3, PadRoles<Rx, Tx, Clk, Rts, NoneT>>
{
    fn cts<P: SomePin>(
        self,
        new_pin: P,
    ) -> Pads<
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::SercomType,
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::P0,
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::P1,
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::P2,
    	<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::P3,
    	PadRoles<
    		<<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::Roles as PadRolesInterface>::Rx,
    		<<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::Roles as PadRolesInterface>::Tx,
    		<<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::Roles as PadRolesInterface>::Clk,
    		<<<Self as ReplacePad<<P::Id as GetPad<S>>::PadNum, P>>::Output as PadsInterface>::Roles as PadRolesInterface>::Rts,
    		<P::Id as GetPad<S>>::PadNum,
		>
	>
    where
        P::Id: GetPad<S>,
        Self: ReplacePad<<P::Id as GetPad<S>>::PadNum, P>,
    {

        let replaced = self.replace_pad(new_pin).access();
        let roles = replaced.roles.access();
        Pads {
            s: replaced.s,
            p0: replaced.p0,
            p1: replaced.p1,
            p2: replaced.p2,
            p3: replaced.p3,
            roles: PadRoles {
                rx: roles.rx,
                tx: roles.tx,
                clk: roles.clk,
                rts: roles.rts,
                cts: PhantomData,
            },
        }
    }
}




fn test(s: Sercom3, pin: Pin<PA16, AlternateD>, pin2: Pin<PA17, AlternateD>, pin3: Pin<PA19, AlternateD>) {
    let _tp = Pads::default(s).io(pin);
}
