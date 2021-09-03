mod parser;

use embedded_hal::digital::v2::OutputPin;
use embedded_hal::digital::v2::InputPin;

use crate::traits::{
    ip::IpAddress,
    wifi::JoinError,
};

use embedded_hal::blocking::spi::{Transfer, Write};
use heapless::{consts::*, String};
use core::fmt::Write as FmtWrite;
use embassy::time::{Duration, Timer};

use parser::{
    JoinResponse,
};

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
const NAK: u8 = 0x15;

macro_rules! command {
    ($size:tt, $($arg:tt)*) => ({
        //let mut c = String::new();
        //c
        let mut c = String::<$size>::new();
        write!(c, $($arg)*).unwrap();
        c.push_str("\r").unwrap();
        c
    })
}

pub struct EsWifiController<SPI, CS, RESET, WAKEUP, READY, E>
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

impl<SPI, CS, RESET, WAKEUP, READY, E> EsWifiController<SPI, CS, RESET, WAKEUP, READY, E>
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

    async fn wakeup(&mut self) {
        self.wakeup.set_low().ok().unwrap();
        Timer::after(Duration::from_millis(50)).await;
        self.wakeup.set_high().ok().unwrap();
        Timer::after(Duration::from_millis(50)).await;
        info!("Wakeup!");
    }

    async fn reset(&mut self) {
        self.reset.set_low().ok().unwrap();
        Timer::after(Duration::from_millis(50)).await;
        self.reset.set_high().ok().unwrap();
        Timer::after(Duration::from_millis(50)).await;
        info!("Reset!");
    }

    pub async fn start(&mut self) -> Result<(), Error<E, CS::Error, RESET::Error, READY::Error>>{
        info!("Starting!");

        self.reset().await;
        self.wakeup().await;

        let mut response = [0; 16];
        let mut pos = 0;

        while self.ready.is_low().map_err(READY)? {
            //info!("waiting for ready");
        }
        loop {
            if pos >= response.len() {
                break;
            }
            let mut chunk = [0x0A, 0x0A];
            self.cs.set_low().map_err(CS)?;
            while self.ready.is_low().map_err(READY)? {}
            self.spi.transfer(&mut chunk).map_err(SPI)?;
            info!("{}", chunk);
            self.cs.set_high().map_err(CS)?;
            // reverse order going from 16 -> 2*8 bits
            if chunk[1] != NAK {
                response[pos] = chunk[1];
                pos += 1;
            }
            if chunk[0] != NAK {
                response[pos] = chunk[0];
                pos += 1;
            }
        }

        let needle = &[b'\r', b'\n', b'>', b' '];

        if !response[0..pos].starts_with(needle) {
            info!(
                "eS-WiFi adapter failed to initialize {:?}",
                &response[0..pos]
            );
        } else {
            // disable verbosity
            // self.send_string(&command!(U8, "MT=1"), &mut response)
            //     .await
            //     .unwrap();
            //self.state = State::Ready;
            info!("eS-WiFi adapter is ready");
        }

        Ok(())
    }

    pub async fn join_wep(&mut self, ssid: &str, password: &str) -> Result<IpAddress, JoinError> {
        info!("Joining!");

        let mut response = [0u8; 1024];

        self.send(&command!(U36, "CB=2").as_bytes(), &mut response)
            .await
            .map_err(|_| JoinError::InvalidSsid)?;

        self.send(&command!(U36, "C1={}", ssid).as_bytes(), &mut response)
            .await
            .map_err(|_| JoinError::InvalidSsid)?;

        self.send(&command!(U72, "C2={}", password).as_bytes(), &mut response)
            .await
            .map_err(|_| JoinError::InvalidPassword)?;

        self.send(&command!(U8, "C3=4").as_bytes(), &mut response)
            .await
            .map_err(|_| JoinError::Unknown)?;

        let response = self
            .send(&command!(U4, "C0").as_bytes(), &mut response)
            .await
            .map_err(|_| JoinError::Unknown)?;

        info!("[[{}]]", response);

        let parse_result = parser::join_response(&response);

        match parse_result {
            Ok((_, response)) => match response {
                JoinResponse::Ok(ip) => Ok(ip),
                JoinResponse::JoinError => Err(JoinError::UnableToAssociate),
            },
            Err(_) => {
                info!("{:?}", &response);
                Err(JoinError::UnableToAssociate)
            }
        }
    }

    async fn send<'a>(
        &'a mut self,
        command: &[u8],
        response: &'a mut [u8],
    ) -> Result<&'a [u8], Error<E, CS::Error, RESET::Error, READY::Error>> {

        while self.ready.is_low().map_err(READY)? {}
        info!("Ready Send...");

        self.cs.set_low().map_err(CS)?;
        self.spi.write(command).map_err(SPI)?;
        self.cs.set_high().map_err(CS)?;

        self.cs.set_low().map_err(CS)?;
        let transfer = self.spi.transfer(response).map_err(SPI)?;
        self.cs.set_high().map_err(CS)?;

        Ok(transfer)
    }

}