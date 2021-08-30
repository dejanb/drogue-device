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

use drogue_device::{
    actors::led::*,
    actors::ticker::*,
    actors::wifi::eswifi::*,
    traits::{wifi::*},
    *};
use embassy_stm32::dbgmcu::Dbgmcu;
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    peripherals:: {PA5, PB13, PB12},
    Peripherals,
};
use embassy::time::Duration;

type Led1Pin = Output<'static, PA5>;
type ENABLE = Output<'static, PB13>;
type RESET = Output<'static, PB12>;

pub struct MyDevice {
//    wifi: EsWifi<ENABLE, RESET>,
    led: ActorContext<'static, Led<Led1Pin>>,
    ticker: ActorContext<'static, Ticker<'static, Led<Led1Pin>>>,
}

static DEVICE: DeviceContext<MyDevice> = DeviceContext::new();

//const WIFI_SSID: &str = include_str!(concat!(env!("OUT_DIR"), "/config/wifi.ssid.txt"));
//const WIFI_PSK: &str = include_str!(concat!(env!("OUT_DIR"), "/config/wifi.password.txt"));
const WIFI_SSID: &str = "zivanac";
const WIFI_PSK: &str = "gosnsljokn";

#[embassy::main]
async fn main(spawner: embassy::executor::Spawner, p: Peripherals) {
    unsafe {
        Dbgmcu::enable_all();
    }

    defmt::info!("Starting up...");

    // WiFi configuration
    // let enable_pin = Output::new(p.PB13, Level::Low, Speed::Low);
    // let reset_pin = Output::new(p.PB12, Level::Low, Speed::Low);



    DEVICE.configure(MyDevice {
        //wifi: EsWifi::new(enable_pin, reset_pin),
        ticker: ActorContext::new(Ticker::new(Duration::from_millis(500), LedMessage::Toggle)),
        led: ActorContext::new(Led::new(Output::new(p.PA5, Level::High, Speed::Low))),
    });

    DEVICE
        .mount(|device| async move {
            // let mut wifi = device.wifi.mount((), spawner);
            // defmt::info!("wifi {} ", WIFI_SSID);
            // wifi.join(Join::Wpa {
            //     ssid: WIFI_SSID.trim_end(),
            //     password: WIFI_PSK.trim_end(),
            // })
            // .await
            // .expect("Error joining wifi");
            // defmt::info!("WiFi network joined");

            let led = device.led.mount((), spawner);
            let ticker = device.ticker.mount(led, spawner);
            ticker
        })
        .await;
}
