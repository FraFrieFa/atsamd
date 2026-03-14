use super::{I2c, I2cBuilder};
use crate::sercom_v2::{Disabled, ProtocolDefinition};

#[test]
fn builder_is_sparse() {
    let runtime = I2cBuilder::new().baud(400_000).smart_mode(true).build();
    assert_eq!(runtime.baud, Some(400_000));
    assert_eq!(runtime.smart_mode, Some(true));
    assert_eq!(runtime.qcmen, None);
}

#[test]
fn i2c_default_builder_entrypoint_is_type_checked() {
    let _mk: fn() -> super::I2cBuilder = super::I2c::default;
}

#[test]
fn field_inventory_mentions_mode_sensitive_fields() {
    let fields = I2c::FIELD_SPECS;
    assert!(fields.iter().any(|field| field.field == "SMEN"));
    assert!(fields.iter().any(|field| field.field == "ACKACT"));
    assert!(fields.iter().any(|field| field.field == "LENEN"));
}

struct TestPads;

impl super::ValidPads for TestPads {
    type Sercom = crate::sercom_v2::Sercom0;
}

#[test]
fn i2c_end_to_end_type_pipeline() {
    let runtime = I2cBuilder::new()
        .baud(400_000)
        .smart_mode(true)
        .sclsm(true)
        .build();

    let resources = super::I2cResources {
        pads: TestPads,
        clock: 48_000_000u32,
        dma: (),
        irqs: (),
    };

    let config = super::I2cConfig::<
        crate::sercom_v2::Sercom0,
        Disabled,
        super::I2cResources<TestPads, u32>,
        super::I2cRuntime,
    >::new(resources, runtime);

    let enabled = config.enable();
    assert_eq!(enabled.runtime().baud, Some(400_000));
    assert_eq!(enabled.resources().clock, 48_000_000u32);
}

#[test]
fn i2c_enable_basic_path_is_type_checked() {
    type Config = super::I2cConfig<
        crate::sercom_v2::Sercom0,
        Disabled,
        super::I2cResources<TestPads, u32>,
        super::I2cRuntime,
    >;

    let _enable_basic: fn(
        Config,
        crate::sercom_v2::Sercom0,
        &crate::sercom_v2::ApbClkCtrl,
    ) -> super::BasicI2c<crate::sercom_v2::Sercom0, TestPads, u32> = Config::enable_basic;
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
fn i2c_enable_from_resource_provider_is_type_checked() {
    type Config = super::I2cConfig<
        crate::sercom_v2::Sercom0,
        Disabled,
        super::I2cResources<TestPads, ()>,
        super::I2cRuntime,
    >;

    let _enable_from: fn(
        Config,
        &mut TestResourceProvider,
    ) -> super::BasicI2c<crate::sercom_v2::Sercom0, TestPads, u32> =
        Config::enable_from::<TestResourceProvider>;
}
