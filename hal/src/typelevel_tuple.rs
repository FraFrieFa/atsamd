//! Utilities for type-level tuples that track presence/absence of fields.
//!
//! The deriving machinery in `atsamd-hal-macros` uses these primitives to wrap
//! structs into tuples where each field is either present or absent at the type
//! level. The tuple stores every field inside a [`Field`] wrapper backed by an
//! `Option`, so runtime moves occur exactly once while the type parameters keep
//! track of ownership.

/// Marker for a present field.
pub struct Present;

/// Marker for an absent field.
pub struct Absent;

use core::marker::PhantomData;

/// Field storage that toggles between `Present` and `Absent` at the type level.
pub struct Field<State, T>(Option<T>, PhantomData<State>);

impl<T> Field<Present, T> {
    /// Construct a present field from an owned value.
    pub fn present(value: T) -> Self {
        Field(Some(value), PhantomData)
    }

    /// Take ownership of the contained value and downgrade to `Absent`.
    pub fn take(self) -> (T, Field<Absent, T>) {
        let Field(value, _) = self;
        (
            value.expect("called take on a present field"),
            Field(None, PhantomData),
        )
    }

    /// Unwrap the value without changing the state.
    pub fn unwrap(self) -> T {
        let Field(value, _) = self;
        value.expect("field must be present")
    }
}

impl<T> Field<Absent, T> {
    /// Construct an absent field placeholder.
    pub fn absent() -> Self {
        Field(None, PhantomData)
    }

    /// Upgrade the field to `Present` by storing the provided value.
    pub fn insert(self, value: T) -> Field<Present, T> {
        let Field(_, _) = self;
        Field(Some(value), PhantomData)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atsamd_hal_macros::TypeLevelTuple;

    #[derive(TypeLevelTuple)]
    struct Example {
        pub foo: u8,
        pub bar: u16,
    }

    #[derive(TypeLevelTuple)]
    struct MockPeripherals {
        pub sercom3: u8,
        pub mclk: u16,
        pub adc0: u32,
    }

    fn take_sercom_resources(
        tuple: MockPeripheralsTuple,
    ) -> (u8, u16, MockPeripheralsTuple<Absent, Absent, Present>) {
        let (sercom3, tuple) = tuple.take_sercom3();
        let (mclk, tuple) = tuple.take_mclk();
        (sercom3, mclk, tuple)
    }

    #[test]
    fn field_takes_and_inserts() {
        let field = Field::<Present, _>::present(42u8);
        let (value, absent) = field.take();
        assert_eq!(value, 42);
        let restored = absent.insert(value);
        assert_eq!(restored.unwrap(), 42);
    }

    #[test]
    fn tuple_round_trip() {
        let example = Example { foo: 1, bar: 2 };
        let tuple = Example::into_tuple(example);
        let (foo, tuple) = tuple.take_foo();
        assert_eq!(foo, 1);
        let tuple = tuple.return_foo(foo);
        let example = tuple.into_struct();
        assert_eq!(example.foo, 1);
        assert_eq!(example.bar, 2);
    }

    #[test]
    fn peripherals_style_extraction_in_function() {
        let peripherals = MockPeripherals {
            sercom3: 3,
            mclk: 48,
            adc0: 7,
        };
        let tuple = MockPeripherals::into_tuple(peripherals);

        let (sercom3, mclk, tuple) = take_sercom_resources(tuple);
        assert_eq!(sercom3, 3);
        assert_eq!(mclk, 48);

        // Unrelated peripherals remain available while extracted fields are absent.
        let (adc0, tuple) = tuple.take_adc0();
        assert_eq!(adc0, 7);

        let tuple = tuple
            .return_sercom3(sercom3)
            .return_mclk(mclk)
            .return_adc0(adc0);
        let restored = tuple.into_struct();
        assert_eq!(restored.sercom3, 3);
        assert_eq!(restored.mclk, 48);
        assert_eq!(restored.adc0, 7);
    }
}
