#![no_std]
#![no_main]
#![macro_use]
#![allow(incomplete_features)]
#![allow(dead_code)]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]
#![feature(concat_idents)]

use defmt_rtt as _;
use panic_probe as _;

use drogue_device::{actors::led::*, actors::ticker::*, *};
use embassy_stm32::dbgmcu::Dbgmcu;
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    peripherals::PA5,
    Peripherals,
};
use embassy::time::Duration;

type Led1Pin = Output<'static, PA5>;

pub struct MyDevice {
    led: ActorContext<'static, Led<Led1Pin>>,
    ticker: ActorContext<'static, Ticker<'static, Led<Led1Pin>>>,
}

static DEVICE: DeviceContext<MyDevice> = DeviceContext::new();

#[embassy::main]
async fn main(spawner: embassy::executor::Spawner, p: Peripherals) {
    unsafe {
        Dbgmcu::enable_all();
    }

    defmt::info!("Starting up...");

    DEVICE.configure(MyDevice {
        ticker: ActorContext::new(Ticker::new(Duration::from_millis(500), LedMessage::Toggle)),
        led: ActorContext::new(Led::new(Output::new(p.PA5, Level::High, Speed::Low))),
    });

    DEVICE
        .mount(|device| async move {
            let led = device.led.mount((), spawner);
            let ticker = device.ticker.mount(led, spawner);
            ticker
        })
        .await;
}
