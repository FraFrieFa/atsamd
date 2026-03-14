use super::{Spi, SpiBuilder};
use crate::sercom_v2::{Disabled, ProtocolDefinition};
use crate::typelevel::NoneT;

#[test]
fn builder_is_sparse() {
    let runtime = SpiBuilder::new().baud(12_000_000).cpol(true).build();
    assert_eq!(runtime.baud, Some(12_000_000));
    assert_eq!(runtime.cpol, Some(true));
    assert_eq!(runtime.cpha, None);
}

#[test]
fn spi_default_builder_entrypoint_is_type_checked() {
    let _mk: fn() -> super::SpiBuilder = super::Spi::default;
}

#[test]
fn field_inventory_mentions_routing_fields() {
    let fields = Spi::FIELD_SPECS;
    assert!(fields.iter().any(|field| field.field == "DIPO"));
    assert!(fields.iter().any(|field| field.field == "DOPO"));
}

struct TestPads;

impl super::ValidPads for TestPads {
    type Sercom = crate::sercom_v2::Sercom0;
    type Capability = super::Duplex;
    type SS = NoneT;
    const DIPO_DOPO: (u8, u8) = (0, 0);
}

#[test]
fn spi_end_to_end_type_pipeline() {
    let runtime = SpiBuilder::new()
        .baud(8_000_000)
        .cpol(false)
        .cpha(false)
        .build();

    let resources = super::SpiResources {
        pads: TestPads,
        clock: 48_000_000u32,
        dma: (),
        irqs: (),
    };

    let config = super::SpiConfig::<
        crate::sercom_v2::Sercom0,
        Disabled,
        super::SpiResources<TestPads, u32>,
        super::SpiRuntime,
    >::new(resources, runtime);

    let enabled = config.enable();
    assert_eq!(enabled.runtime().baud, Some(8_000_000));
    assert_eq!(enabled.resources().clock, 48_000_000u32);
}

#[test]
fn spi_enable_basic_path_is_type_checked() {
    type Config = super::SpiConfig<
        crate::sercom_v2::Sercom0,
        Disabled,
        super::SpiResources<TestPads, u32>,
        super::SpiRuntime,
    >;

    let _enable_basic: fn(
        Config,
        crate::sercom_v2::Sercom0,
        &crate::sercom_v2::ApbClkCtrl,
    ) -> super::BasicSpi<crate::sercom_v2::Sercom0, super::Duplex, TestPads, u32> =
        Config::enable_basic;
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
fn spi_enable_from_resource_provider_is_type_checked() {
    type Config = super::SpiConfig<
        crate::sercom_v2::Sercom0,
        Disabled,
        super::SpiResources<TestPads, ()>,
        super::SpiRuntime,
    >;

    let _enable_from: fn(
        Config,
        &mut TestResourceProvider,
    ) -> super::BasicSpi<crate::sercom_v2::Sercom0, super::Duplex, TestPads, u32> =
        Config::enable_from::<TestResourceProvider>;
}
