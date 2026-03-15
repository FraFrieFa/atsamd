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
#[test]
fn usart_from_peripherals_and_two_pins_requires_clock_channel() {
    use crate::clock::v2::{dfll::DfllId, gclk::EnabledGclk0, gclk::Gclk0Id, pclk::Pclk};
    use crate::sercom_v2::IntoPeriphV2ClockedSercom3;

    type RxPin = crate::gpio::Pin<crate::gpio::PA23, crate::gpio::Reset>;
    type TxPin = crate::gpio::Pin<crate::gpio::PA22, crate::gpio::Reset>;
    type Sercom3Pclk = Pclk<crate::sercom::Sercom3, Gclk0Id>;
    type Gclk0 = EnabledGclk0<DfllId>;

    type ResolvedPads = super::InternalPads<
        crate::sercom_v2::pads::Pads<
            crate::sercom_v2::Pad<crate::sercom_v2::Sercom3, crate::gpio::PA23>,
            crate::sercom_v2::Pad<crate::sercom_v2::Sercom3, crate::gpio::PA22>,
            crate::typelevel::NoneT,
            crate::typelevel::NoneT,
        >,
        super::Roles<
            crate::sercom_v2::Pad<crate::sercom_v2::Sercom3, crate::gpio::PA23>,
            crate::sercom_v2::Pad<crate::sercom_v2::Sercom3, crate::gpio::PA22>,
            crate::typelevel::NoneT,
            crate::typelevel::NoneT,
            crate::typelevel::NoneT,
        >,
    >;

    type ExpectedUsart =
        super::BasicUsart<crate::sercom_v2::Sercom3, super::Duplex, ResolvedPads, Sercom3Pclk>;

    let _build_from_ref:
        fn(crate::pac::Peripherals, Sercom3Pclk, Gclk0, RxPin, TxPin) -> ExpectedUsart =
        |peripherals, pclk, gclk0, rx, tx| {
            let mut pac_v2 = peripherals.into_periphv2_clocked_sercom3(pclk);
            super::Usart::reset_config()
                .rx(rx)
                .tx(tx)
                .baud(115_200)
                .enable(&mut pac_v2, &gclk0)
        };
}
