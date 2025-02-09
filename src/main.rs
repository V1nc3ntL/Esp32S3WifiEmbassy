#![no_std]
#![no_main]

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");
use core::primitive::str;

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

mod configuration {
    pub mod hardware;
    pub mod http;
}
mod execution {
    pub mod hardware;
    pub mod http;
}
mod peripherals {
    pub mod pmu;
}

use crate::configuration::hardware::*;
use crate::execution::hardware::*;
use crate::execution::http::*;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) -> ! {
    // TODO : actually handle the None case

    let (stack, runner, controller) = get_runner_controller_stack().unwrap();

    spawner.spawn(net_task(runner)).ok();

    spawner.spawn(connection(controller)).ok();

    spawner.spawn(launch_dhcp(*stack)).ok();

    stack.wait_config_up().await;
    RX_BUFFERS_CELL
        .iter()
        .zip(TX_BUFFERS_CELL.iter())
        .zip(HTTP_SOCKETS_CELL.iter())
        .for_each(|iter| {
            let rx = iter.0 .0.init_with(|| [0; RX_BUFFER_SIZE]);
            let tx = iter.0 .1.init_with(|| [0; TX_BUFFER_SIZE]);
            spawner
                .spawn(answer_to_http(
                    iter.1.init_with(|| TcpSocket::new(*stack, rx, tx)),
                ))
                .ok();
        });
    loop {
        Timer::after(embassy_time::Duration::from_millis(1_000)).await;
    }
}

async fn wait_for<F>(check: F, delay_ms: u64, message: &str)
where
    F: Fn() -> bool,
{
    while !check() {
        println!("{}", message);
        Timer::after(embassy_time::Duration::from_millis(delay_ms)).await;
    }
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    println!("start connection task");
    if esp_wifi::wifi::wifi_state() == WifiState::StaConnected {
        // wait until we're no longer connected
        controller.wait_for_event(WifiEvent::StaDisconnected).await;
        Timer::after(embassy_time::Duration::from_millis(5000)).await;
    }

    if !matches!(controller.is_started(), Ok(true)) {
        let client_config = Configuration::Client(ClientConfiguration {
            ssid: SSID.try_into().unwrap(),
            password: PASSWORD.try_into().unwrap(),
            ..Default::default()
        });
        match controller.set_configuration(&client_config) {
            Ok(()) => println!("Configuring wifi"),
            Err(_) => {
                println!("Could not set wifi configuration");
                return;
            }
        }
        match controller.start() {
            Ok(()) => println!("Wifi started"),
            Err(_) => {
                println!("Could not start wifi");
                return;
            }
        }
    }
    while let Err(e) = controller.connect() {
        println!("Failed to connect to wifi: {e:?}");
        Timer::after(embassy_time::Duration::from_millis(1000)).await;
    }
    println!("Wifi connected!");
}

#[embassy_executor::task]
async fn launch_dhcp(stack: Stack<'static>) {
    println!("Launching DHCP");

    wait_for(|| stack.is_link_up(), 2000, "Waiting for Stack").await;

    println!("Waiting to get IP address...");
    loop {
        if let Some(config) = stack.config_v4() {
            println!("Got IP: {}", config.address);
            break;
        }
        Timer::after(embassy_time::Duration::from_millis(1000)).await;
    }
}
/// The size of the buffer to read incoming request
const READ_BUFFER_SIZE: usize = 1024;
/// The buffer memory that is used
static READ_BUFFER: static_cell::StaticCell<[u8; READ_BUFFER_SIZE]> =
    const { static_cell::StaticCell::new() };

#[embassy_executor::task]
async fn answer_to_http(socket: &'static mut TcpSocket<'static>) {
    socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));
    let buf = READ_BUFFER.init_with(|| [0; READ_BUFFER_SIZE]);

    loop {
        const HTTP_PORT: u16 = 80;
        match socket
            .accept(IpListenEndpoint {
                addr: None,
                port: HTTP_PORT,
            })
            .await
        {
            Ok(_v) => loop {
                println!("Accepted");
                match socket.read(buf).await {
                    Ok(0) => {
                        println!("read EOF");
                        socket.close();
                        break;
                    }
                    Ok(n) => {
                        println!("received {} bytes", n);

                        match &buf.get(..n) {
                            Some(x) => {
                                match handle_request(socket, x).await {
                                    Ok(()) => continue,
                                    Err(e) => println!("Error when responding to request: {:?}", e),
                                };
                            }
                            None => break,
                        }
                    }
                    Err(e) => {
                        println!("read error: {:?}", e);
                        break;
                    }
                };
            },
            Err(e) => {
                println!("accept error: {:?}", e);
                println!("socket state: {:?}", socket.state());
                break;
            }
        };
    }
}

#[embassy_executor::task]
async fn net_task(
    runner: &'static mut Runner<'static, &'static mut WifiDevice<'static, WifiStaDevice>>,
) {
    runner.run().await;
}
