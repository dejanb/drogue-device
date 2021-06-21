#![no_std]
#![no_main]
#![macro_use]
#![allow(incomplete_features)]
#![allow(dead_code)]
#![feature(generic_associated_types)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![feature(concat_idents)]

use defmt_rtt as _;
use panic_probe as _;

use drogue_device::{actors::ticker::*, actors::led::*, *};
use embassy_stm32::{
    gpio::{Level, Output},
    interrupt,
    peripherals::PA5,
    Peripherals,
};

use embassy::time::Duration;
use stm32l4::stm32l4x2 as pac;

type Led1Pin = Output<'static, PA5>;

pub struct MyDevice {
    led: ActorContext<'static, Led<Led1Pin>>,
    ticker: ActorContext<'static, Ticker<'static, Led<Led1Pin>>>,
}

static DEVICE: DeviceContext<MyDevice> = DeviceContext::new();

#[embassy::main]
async fn main(spawner: embassy::executor::Spawner, p: Peripherals) {
    let pp = pac::Peripherals::take().unwrap();

    pp.DBGMCU.cr.modify(|_, w| {
        w.dbg_sleep().set_bit();
        w.dbg_standby().set_bit();
        w.dbg_stop().set_bit()
    });

    pp.RCC.ahb1enr.modify(|_, w| w.dma1en().set_bit());

    pp.RCC.ahb2enr.modify(|_, w| {
        w.gpioaen().set_bit();
        w.gpioben().set_bit();
        w.gpiocen().set_bit();
        w.gpioden().set_bit();
        w.gpioeen().set_bit();
        w
    });

    defmt::info!("Starting up...");

    DEVICE.configure(MyDevice {
        ticker: ActorContext::new(Ticker::new(Duration::from_millis(500), LedMessage::Toggle)),
        led: ActorContext::new(Led::new(Output::new(p.PA5, Level::High))),
    });

    DEVICE.mount(|device| {
        let led = device.led.mount((), spawner);
        let ticker = device.ticker.mount(led, spawner);
        ticker.notify(TickerCommand::Start).unwrap();
    });
}
