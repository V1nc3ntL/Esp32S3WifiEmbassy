//! Embassy DHCP Example
//!
//!
//! Set SSID and PASSWORD env variable before running this example.
//!
//! This gets an ip address via DHCP then performs an HTTP get request to some "random" server
//!
//! Because of the huge task-arena size configured this won't work on ESP32-S2
//! When using USB-SERIAL-JTAG you have to activate the feature `phy-enable-usb` in the esp-wifi crate.

//% FEATURES: embassy embassy-generic-timers esp-wifi esp-wifi/async esp-wifi/embassy-net esp-wifi/wifi-default esp-wifi/wifi esp-wifi/utils
//% CHIPS: esp32 esp32s2 esp32s3 esp32c2 esp32c3 esp32c6
#![no_std]
#![no_main]
const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");
use core::primitive::str;
mod configuration;
use embassy_executor::Spawner;
use embassy_net::{tcp::TcpSocket, IpListenEndpoint, Runner, Stack};
use embassy_time::Timer;
use esp_alloc as _;
use esp_backtrace as _;
use esp_println::println;
use esp_wifi::wifi::{
    ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiStaDevice,
    WifiState,
};

pub mod application_layer;
pub mod hardware;
use application_layer::handle_request;
use hardware::get_runner_controller_stack;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) -> ! {
    let (stack, runner, controller) = get_runner_controller_stack();
    spawner.spawn(net_task(runner)).ok();

    spawner.spawn(connection(controller)).ok();

    spawner.spawn(launch_dhcp(stack)).ok();

    stack.wait_config_up().await;
    configuration::RX_BUFFERS_CELL
        .iter()
        .zip(configuration::TX_BUFFERS_CELL.iter())
        .zip(configuration::HTTP_SOCKETS_CELL.iter())
        .for_each(|iter| {
            let rx = iter.0 .0.init_with(|| [0; configuration::RX_BUFFER_SIZE]);
            let tx = iter.0 .1.init_with(|| [0; configuration::TX_BUFFER_SIZE]);
            spawner
                .spawn(answer_to_http(
                    iter.1.uninit().write(TcpSocket::new(*stack, rx, tx)),
                ))
                .ok();
        });
    loop {
        Timer::after(embassy_time::Duration::from_millis(1_000)).await;
    }
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    println!("start connection task");
    println!("Device capabilities: {:?}", controller.capabilities());
    if esp_wifi::wifi::wifi_state() == WifiState::StaConnected {
        // wait until we're no longer connected
        controller.wait_for_event(WifiEvent::StaDisconnected).await;
        Timer::after(embassy_time::Duration::from_millis(5000)).await
    }

    if !matches!(controller.is_started(), Ok(true)) {
        let client_config = Configuration::Client(ClientConfiguration {
            ssid: SSID.try_into().unwrap(),
            password: PASSWORD.try_into().unwrap(),
            ..Default::default()
        });
        controller.set_configuration(&client_config).unwrap();
        println!("Starting wifi");
        controller.start().unwrap();
        println!("Wifi started!");
    }
    println!("About to connect...");

    loop {
        match controller.connect() {
            Ok(_) => {
                println!("Wifi connected!");
                break;
            }
            Err(e) => {
                println!("Failed to connect to wifi: {e:?}");
                Timer::after(embassy_time::Duration::from_millis(1000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn launch_dhcp(stack: &'static Stack<'static>) {
    println!("Launching DHCP");
    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(embassy_time::Duration::from_millis(500)).await;
    }

    println!("Waiting to get IP address...");
    loop {
        if let Some(config) = stack.config_v4() {
            println!("Got IP: {}", config.address);
            break;
        }
        Timer::after(embassy_time::Duration::from_millis(1000)).await;
    }
}

#[embassy_executor::task]
async fn answer_to_http(socket: &'static mut TcpSocket<'static>) {
    socket.set_timeout(Some(embassy_time::Duration::from_secs(2)));
    loop {
        const HTTP_PORT: u16 = 80;
        match socket
            .accept(IpListenEndpoint {
                addr: None,
                port: HTTP_PORT,
            })
            .await
        {
            Ok(_v) => {
                let mut buf: [u8; 1024] = [0; 1024];
                loop {
                    match socket.read(&mut buf).await {
                        Ok(0) => {
                            println!("read EOF");
                            break;
                        }
                        Ok(n) => {
                            println!("received {} bytes", n);
                            match handle_request(socket, &buf, n).await {
                                Ok(()) => continue,
                                Err(e) => println!("Error when responding to request: {:?}", e),
                            };
                        }
                        Err(e) => {
                            println!("read error: {:?}", e);
                            break;
                        }
                    };
                }
            }
            Err(e) => {
                println!("accept error: {:?}", e);
                break;
            }
        };
    }
}

#[embassy_executor::task]
async fn net_task(
    runner: &'static mut Runner<'static, &'static mut WifiDevice<'static, WifiStaDevice>>,
) {
    runner.run().await
}
