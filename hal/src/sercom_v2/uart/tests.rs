use super::{Usart, UsartBuilder};
use crate::sercom_v2::{Disabled, ProtocolDefinition};
use crate::typelevel::NoneT;

#[test]
fn builder_is_sparse() {
    let runtime = UsartBuilder::new().dord(true).baud(115_200).build();
    assert_eq!(runtime.dord, Some(true));
    assert_eq!(runtime.baud, Some(115_200));
    assert_eq!(runtime.sampr, None);
}

#[test]
fn field_inventory_mentions_mode_sensitive_fields() {
    let fields = Usart::FIELD_SPECS;
    assert!(fields.iter().any(|field| field.field == "FORM"));
    assert!(fields.iter().any(|field| field.field == "RXPO"));
    assert!(fields.iter().any(|field| field.field == "TXPO"));
    assert!(fields.iter().any(|field| field.field == "HDRDLY"));
}

#[test]
fn arithmetic_baud_register_matches_expected_baseline() {
    let baud = super::calculate_baud_asynchronous_arithm(115_200, 48_000_000, 16);
    assert_eq!(baud, 63_019);
}

struct TestPads;

impl super::ValidPads for TestPads {
    const RXPO: u8 = 1;
    const TXPO: u8 = 0;
    type Capability = super::Duplex;
    type Sercom = crate::sercom_v2::Sercom0;
    type CTS = NoneT;
}

#[test]
fn usart_end_to_end_type_pipeline() {
    let runtime = UsartBuilder::new()
        .baud(115_200)
        .dord(false)
        .runstdby(false)
        .build();

    let resources = super::UsartResources {
        pads: TestPads,
        clock: 48_000_000u32,
        dma: (),
        irqs: (),
    };

    let config = super::UsartConfig::<
        crate::sercom_v2::Sercom0,
        Disabled,
        super::UsartResources<TestPads, u32>,
        super::UsartRuntime,
    >::new(resources, runtime);

    let enabled = config.enable();
    assert_eq!(enabled.runtime().baud, Some(115_200));
    assert_eq!(enabled.resources().clock, 48_000_000u32);
}

#[test]
fn usart_enable_basic_path_is_type_checked() {
    type Config = super::UsartConfig<
        crate::sercom_v2::Sercom0,
        Disabled,
        super::UsartResources<TestPads, u32>,
        super::UsartRuntime,
    >;

    let _enable_basic: fn(
        Config,
        crate::sercom_v2::Sercom0,
        &crate::sercom_v2::ApbClkCtrl,
    ) -> super::BasicUsart<
        crate::sercom_v2::Sercom0,
        super::Duplex,
        TestPads,
        u32,
    > = Config::enable_basic;
}

struct TestResourceProvider;

impl crate::sercom_v2::TakeSercom<crate::sercom_v2::Sercom0> for TestResourceProvider {
    fn take_sercom(&mut self) -> crate::sercom_v2::Sercom0 {
        panic!("type-check only")
    }
}

impl crate::sercom_v2::TakeSercomCoreClock<crate::sercom_v2::Sercom0> for TestResourceProvider {
    type Clock = u32;

    fn take_sercom_core_clock(&mut self) -> Self::Clock {
        panic!("type-check only")
    }
}

impl crate::sercom_v2::HasApbClkCtrl for TestResourceProvider {
    fn apb_clk_ctrl(&self) -> &crate::sercom_v2::ApbClkCtrl {
        panic!("type-check only")
    }
}

#[test]
fn usart_enable_from_resource_provider_is_type_checked() {
    type Config = super::UsartConfig<
        crate::sercom_v2::Sercom0,
        Disabled,
        super::UsartResources<TestPads, ()>,
        super::UsartRuntime,
    >;

    let _enable_from: fn(
        Config,
        &mut TestResourceProvider,
    ) -> super::BasicUsart<
        crate::sercom_v2::Sercom0,
        super::Duplex,
        TestPads,
        u32,
    > = Config::enable_from::<TestResourceProvider>;
}

#[cfg(any(
    feature = "samd21e",
    feature = "samd21g",
    feature = "samd21j",
    feature = "samd21el",
    feature = "samd21gl"
))]
#[test]
fn usart_default_builder_rx_tx_is_type_checked() {
    type ExpectedBuilder = super::AutoUsartBuilder<
        crate::gpio::Pin<crate::gpio::PA11, crate::gpio::Reset>,
        crate::gpio::Pin<crate::gpio::PA10, crate::gpio::Reset>,
    >;

    let _build: fn(
        crate::gpio::Pin<crate::gpio::PA11, crate::gpio::Reset>,
        crate::gpio::Pin<crate::gpio::PA10, crate::gpio::Reset>,
    ) -> ExpectedBuilder = |rx, tx| super::Usart::default().rx(rx).tx(tx);
}

#[cfg(any(
    feature = "samd21e",
    feature = "samd21g",
    feature = "samd21j",
    feature = "samd21el",
    feature = "samd21gl"
))]
#[test]
fn auto_builder_accepts_unconfigured_pins_and_infers_sercom() {
    type ExpectedConfig = super::UsartConfig<
        crate::sercom_v2::Sercom0,
        Disabled,
        super::UsartResources<
            super::InternalPads<
                crate::sercom_v2::pads::Pads<
                    crate::sercom_v2::Pad<crate::sercom_v2::Sercom0, crate::gpio::PA11>,
                    crate::sercom_v2::Pad<crate::sercom_v2::Sercom0, crate::gpio::PA10>,
                    crate::typelevel::NoneT,
                    crate::typelevel::NoneT,
                >,
                super::Roles<
                    crate::sercom_v2::Pad<crate::sercom_v2::Sercom0, crate::gpio::PA11>,
                    crate::sercom_v2::Pad<crate::sercom_v2::Sercom0, crate::gpio::PA10>,
                    crate::typelevel::NoneT,
                    crate::typelevel::NoneT,
                    crate::typelevel::NoneT,
                >,
            >,
            u32,
        >,
        super::UsartRuntime,
    >;
    type Builder = super::AutoUsartBuilder<
        crate::gpio::Pin<crate::gpio::PA11, crate::gpio::Reset>,
        crate::gpio::Pin<crate::gpio::PA10, crate::gpio::Reset>,
    >;
    let _map_fn: fn(Builder) -> ExpectedConfig =
        |builder| builder.baud(115_200).to_config_first(48_000_000);
}

#[cfg(any(
    feature = "samd21e",
    feature = "samd21g",
    feature = "samd21j",
    feature = "samd21el",
    feature = "samd21gl"
))]
#[test]
fn auto_builder_rx_tx_enable_basic_path_is_type_checked() {
    type Builder = super::AutoUsartBuilder<
        crate::gpio::Pin<crate::gpio::PA11, crate::gpio::Reset>,
        crate::gpio::Pin<crate::gpio::PA10, crate::gpio::Reset>,
    >;

    type ResolvedPads = super::InternalPads<
        crate::sercom_v2::pads::Pads<
            crate::sercom_v2::Pad<crate::sercom_v2::Sercom0, crate::gpio::PA11>,
            crate::sercom_v2::Pad<crate::sercom_v2::Sercom0, crate::gpio::PA10>,
            crate::typelevel::NoneT,
            crate::typelevel::NoneT,
        >,
        super::Roles<
            crate::sercom_v2::Pad<crate::sercom_v2::Sercom0, crate::gpio::PA11>,
            crate::sercom_v2::Pad<crate::sercom_v2::Sercom0, crate::gpio::PA10>,
            crate::typelevel::NoneT,
            crate::typelevel::NoneT,
            crate::typelevel::NoneT,
        >,
    >;

    type ExpectedUsart =
        super::BasicUsart<crate::sercom_v2::Sercom0, super::Duplex, ResolvedPads, u32>;

    let _construct_and_enable: fn(
        Builder,
        crate::sercom_v2::Sercom0,
        &crate::sercom_v2::ApbClkCtrl,
    ) -> ExpectedUsart = |builder, sercom, apb| {
        builder
            .baud(115_200)
            .to_config_first(48_000_000)
            .enable_basic(sercom, apb)
    };
}

#[cfg(any(
    feature = "samd21e",
    feature = "samd21g",
    feature = "samd21j",
    feature = "samd21el",
    feature = "samd21gl"
))]
#[test]
fn auto_builder_rx_tx_enable_from_is_type_checked() {
    type Builder = super::AutoUsartBuilder<
        crate::gpio::Pin<crate::gpio::PA11, crate::gpio::Reset>,
        crate::gpio::Pin<crate::gpio::PA10, crate::gpio::Reset>,
    >;

    type ResolvedPads = super::InternalPads<
        crate::sercom_v2::pads::Pads<
            crate::sercom_v2::Pad<crate::sercom_v2::Sercom0, crate::gpio::PA11>,
            crate::sercom_v2::Pad<crate::sercom_v2::Sercom0, crate::gpio::PA10>,
            crate::typelevel::NoneT,
            crate::typelevel::NoneT,
        >,
        super::Roles<
            crate::sercom_v2::Pad<crate::sercom_v2::Sercom0, crate::gpio::PA11>,
            crate::sercom_v2::Pad<crate::sercom_v2::Sercom0, crate::gpio::PA10>,
            crate::typelevel::NoneT,
            crate::typelevel::NoneT,
            crate::typelevel::NoneT,
        >,
    >;

    type ExpectedUsart =
        super::BasicUsart<crate::sercom_v2::Sercom0, super::Duplex, ResolvedPads, u32>;

    let _construct_and_enable_from: fn(Builder, &mut TestResourceProvider) -> ExpectedUsart =
        |builder, resources| builder.baud(115_200).to_config().enable_from(resources);
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
#[test]
fn auto_builder_samd5x_three_pins_infers_sercom5() {
    type ExpectedConfig = super::UsartConfig<
        crate::sercom_v2::Sercom5,
        Disabled,
        super::UsartResources<
            super::Pads<
                crate::sercom_v2::Pad<crate::sercom_v2::Sercom5, crate::gpio::PA22>,
                crate::sercom_v2::Pad<crate::sercom_v2::Sercom5, crate::gpio::PA23>,
                crate::sercom_v2::Pad<crate::sercom_v2::Sercom5, crate::gpio::PA24>,
                crate::typelevel::NoneT,
            >,
            u32,
        >,
        super::UsartRuntime,
    >;

    type Builder = super::AutoUsartBuilder<
        crate::gpio::Pin<crate::gpio::PA22, crate::gpio::Reset>,
        crate::gpio::Pin<crate::gpio::PA23, crate::gpio::Reset>,
        crate::typelevel::NoneT,
        crate::gpio::Pin<crate::gpio::PA24, crate::gpio::Reset>,
        crate::typelevel::NoneT,
    >;

    let _map_fn: fn(Builder) -> ExpectedConfig =
        |builder| builder.baud(115_200).to_config_first(48_000_000);
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
#[test]
fn auto_builder_enable_from_pac_uses_periphv2_tuple_and_infers_sercom3() {
    type Builder = super::AutoUsartBuilder<
        crate::gpio::Pin<crate::gpio::PA23, crate::gpio::Reset>,
        crate::gpio::Pin<crate::gpio::PA22, crate::gpio::Reset>,
    >;

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
        super::BasicUsart<crate::sercom_v2::Sercom3, super::Duplex, ResolvedPads, u32>;

    let _construct_and_enable_from_pac: fn(Builder, crate::pac::Peripherals) -> ExpectedUsart =
        |builder, peripherals| {
            builder.baud(115_200).to_config().enable_from_pac(
                crate::sercom_v2::IntoPeriphV2::into_periphv2(peripherals),
                48_000_000u32,
            )
        };
}
