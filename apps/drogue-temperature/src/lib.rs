#![no_std]
#![macro_use]
#![feature(type_alias_impl_trait)]
#![feature(generic_associated_types)]

pub(crate) mod fmt;

use core::fmt::Write;
use core::future::Future;
use drogue_device::{
    actors::button::{ButtonEvent, FromButtonEvent},
    actors::sensors::SensorMonitor,
    clients::http::*,
    domain::{temperature::Celsius, SensorAcquisition},
    traits::{ip::*, tcp::*},
    Actor, Address, Inbox,
};
use heapless::String;
//use drogue_device::domain::temperature::Temperature;

#[derive(Clone)]
pub struct GeoLocation {
    pub lon: f64,
    pub lat: f64,
}

#[derive(Clone)]
pub struct SensorData {
    pub data: SensorAcquisition<Celsius>,
    pub location: Option<GeoLocation>,
}

pub enum Command {
    Send,
    Update(SensorData),
}

impl<S> FromButtonEvent<Command> for App<S>
where
    S: TcpSocket + 'static,
{
    fn from(event: ButtonEvent) -> Option<Command> {
        match event {
            ButtonEvent::Pressed => None,
            ButtonEvent::Released => Some(Command::Send),
        }
    }
}

pub struct App<S>
where
    S: TcpSocket + 'static,
{
    ip: IpAddress,
    port: u16,
    username: &'static str,
    password: &'static str,
    socket: Option<S>,
}

impl<S> App<S>
where
    S: TcpSocket + 'static,
{
    pub fn new(ip: IpAddress, port: u16, username: &'static str, password: &'static str) -> Self {
        Self {
            ip,
            port,
            username,
            password,
            socket: None,
        }
    }
}

impl<S> Actor for App<S>
where
    S: TcpSocket + 'static,
{
    type Configuration = S;
    #[rustfmt::skip]
    type Message<'m> where S: 'm = Command;

    #[rustfmt::skip]
    type OnMountFuture<'m, M> where S: 'm, M: 'm = impl Future<Output = ()> + 'm;
    fn on_mount<'m, M>(
        &'m mut self,
        config: Self::Configuration,
        _: Address<'static, Self>,
        inbox: &'m mut M,
    ) -> Self::OnMountFuture<'m, M>
    where
        M: Inbox<'m, Self> + 'm,
    {
        self.socket.replace(config);
        async move {
            let mut sensor_data: Option<SensorData> = None;
            loop {
                match inbox.next().await {
                    Some(mut m) => match m.message() {
                        Command::Update(t) => {
                            //trace!("Updating current app temperature measurement: {}", t);
                            sensor_data.replace(t.clone());
                        }
                        Command::Send => match &sensor_data {
                            Some(t) => {
                                info!("Sending temperature measurement");
                                let socket = self.socket.as_mut().unwrap();
                                let mut client = HttpClient::new(
                                    socket,
                                    self.ip,
                                    self.port,
                                    self.username,
                                    self.password,
                                );

                                let mut tx: String<256> = String::new();
                                if let Some(loc) = &t.location {
                                    write!(
                                        tx,
                                        "{{\"geoloc\"{{\"lat\": {}, \"lon\": {}}}, \"temp\": {}, \"hum\": {}}}",
                                        loc.lat,
                                        loc.lon,
                                        t.data.temperature, t.data.relative_humidity
                                    )
                                    .unwrap();
                                } else {
                                    write!(
                                        tx,
                                        "{{\"temp\": {}, \"hum\": {}}}",
                                        t.data.temperature, t.data.relative_humidity
                                    )
                                    .unwrap();
                                }
                                let mut rx_buf = [0; 1024];
                                let response = client
                                    .request(
                                        Request::post()
                                            .path("/v1/foo")
                                            .payload(tx.as_bytes())
                                            .content_type(ContentType::ApplicationJson),
                                        &mut rx_buf[..],
                                    )
                                    .await;
                                socket.close().await;
                                match response {
                                    Ok(response) => {
                                        info!("Response status: {:?}", response.status);
                                        if let Some(payload) = response.payload {
                                            let s = core::str::from_utf8(payload).unwrap();
                                            info!("Payload: {}", s);
                                        } else {
                                            info!("No response body");
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Error doing HTTP request: {:?}", e);
                                    }
                                }
                            }
                            None => {
                                info!("No temperature value found, not sending measurement");
                            }
                        },
                    },
                    _ => {}
                }
            }
        }
    }
}

pub struct AppAddress<S: TcpSocket + 'static> {
    address: Address<'static, App<S>>,
}

impl<S: TcpSocket> From<Address<'static, App<S>>> for AppAddress<S> {
    fn from(address: Address<'static, App<S>>) -> Self {
        Self { address }
    }
}

impl<S: TcpSocket> SensorMonitor<Celsius> for AppAddress<S> {
    fn notify(&self, value: SensorAcquisition<Celsius>) {
        // Ignore channel full error
        let _ = self.address.notify(Command::Update(SensorData {
            data: value,
            location: None,
        }));
    }
}
