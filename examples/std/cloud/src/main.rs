#![macro_use]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use drogue_device::{domain::temperature::Celsius, drivers::tcp::std::*, *};
use drogue_temperature::*;
use embassy::time::Duration;
use rand::rngs::OsRng;

pub struct StdBoard;

impl TemperatureBoard for StdBoard {
    type Network = StdTcpClient;
    type TemperatureScale = Celsius;
    type SensorReadyIndicator = AlwaysReady;
    type Sensor = FakeSensor;
    type SendTrigger = TimeTrigger;
    type Rng = OsRng;
}

static DEVICE: DeviceContext<TemperatureDevice<StdBoard>> = DeviceContext::new();

#[embassy::main]
async fn main(spawner: embassy::executor::Spawner) {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .format_timestamp_nanos()
        .init();

    DEVICE
        .configure(TemperatureDevice::new())
        .mount(
            spawner,
            OsRng,
            TemperatureBoardConfig {
                send_trigger: TimeTrigger(Duration::from_secs(10)),
                sensor: FakeSensor(22.0),
                sensor_ready: AlwaysReady,
                network: StdTcpClient,
            },
        )
        .await;
}
