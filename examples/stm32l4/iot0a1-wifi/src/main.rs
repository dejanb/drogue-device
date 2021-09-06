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
//    actors::wifi::eswifi::*,
//    traits::{wifi::*},
    *};
use embassy_stm32::dbgmcu::Dbgmcu;
use embassy_stm32::{
    gpio::{Level, Input, Output, Speed, Pull},
    peripherals:: {PA5, PB13, PB12},
    Peripherals,
};
use embassy_stm32::spi::{Config, Spi};
use embassy_stm32::time::Hertz;
use defmt::*;
use embassy_stm32::dma::NoDma;
use embedded_hal::digital::v2::{InputPin, OutputPin};

use cortex_m::prelude::_embedded_hal_blocking_spi_Transfer;
use drogue_device::drivers::wifi::eswifi::EsWifiController;


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

#[embassy::main]
async fn main(_spawner: embassy::executor::Spawner, p: Peripherals) {
    unsafe {
        Dbgmcu::enable_all();
    }

    defmt::info!("Starting up...");

    let mut spi = Spi::new(
        p.SPI3,
        p.PC10,
        p.PC12,
        p.PC11,
        NoDma,
        NoDma,
        Hertz(1_000_000),
        Config::default(),
    );

    //let mut cs = Output::new(p.PE0, Level::High, Speed::VeryHigh);

    let _boot = Output::new(p.PB12, Level::Low, Speed::VeryHigh);
    let wake = Output::new(p.PB13, Level::High, Speed::VeryHigh);
    let reset = Output::new(p.PE8, Level::High, Speed::VeryHigh);
    let mut cs = Output::new(p.PE0, Level::High, Speed::VeryHigh);
    let ready = Input::new(p.PE1, Pull::Up);

    // let mut wifi = XEsWifiController::new(spi, reset, wake, cs, ready);
    // wifi.testing().await;

    for n in 1..20 {
        let mut buf = [0x0Au8; 4];

        defmt::unwrap!(cs.set_low());
        defmt::unwrap!(spi.transfer(&mut buf));
        defmt::unwrap!(cs.set_high());

        defmt::info!("xfer {=[u8]:x}", buf);
    }

}





#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<SPI, CS, RESET, READY> {
    Uninformative,
    VersionMismatch(u8),
    CS(CS),
    Reset(RESET),
    SPI(SPI),
    READY(READY),
    Transmitting,
}

use Error::*;
use embedded_hal::blocking::spi::{Transfer, Write};

pub struct XEsWifiController<SPI, CS, RESET, WAKEUP, READY, E>
where
    SPI: Transfer<u8, Error = E> + Write<u8, Error = E>,
    CS: OutputPin + 'static,
    RESET: OutputPin + 'static,
    WAKEUP: OutputPin + 'static,
    READY: InputPin + 'static,
    E: 'static,
{
    spi: SPI,
    cs: CS,
    reset: RESET,
    wakeup: WAKEUP,
    ready: READY,
}

impl<SPI, CS, RESET, WAKEUP, READY, E> XEsWifiController<SPI, CS, RESET, WAKEUP, READY, E>
where
    SPI: Transfer<u8, Error = E> + Write<u8, Error = E>,
    CS: OutputPin + 'static,
    RESET: OutputPin + 'static,
    WAKEUP: OutputPin + 'static,
    READY: InputPin + 'static,
    E: 'static,
{
    pub fn new(
        spi: SPI,
        cs: CS,
        reset: RESET,
        wakeup: WAKEUP,
        ready: READY,
    ) -> Self {
        Self {
            spi,
            cs,
            reset,
            wakeup,
            ready,
        }
    }

    pub async fn testing(&mut self) -> Result<(), Error<E, CS::Error, RESET::Error, READY::Error>>{
        for n in 1..20 {
            let mut buf = [0x0Au8; 4];

            self.cs.set_low().map_err(CS)?;
            self.spi.transfer(&mut buf).map_err(SPI)?;
            self.cs.set_high().map_err(CS)?;

            defmt::info!("xfer {=[u8]:x}", buf);
        }

        Ok(())

    }
}