use embassy_net::{new, Runner, Stack, StackResources};
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{prelude::*, rng::Rng, timer::timg::TimerGroup};
use esp_wifi::wifi::{WifiController, WifiDevice, WifiStaDevice};

static NETWORK_STACK_RESSOURCES_CELL: static_cell::StaticCell<StackResources<3>> =
    static_cell::StaticCell::new();
static NETWORK_STACK_CELL: static_cell::StaticCell<(
    Stack<'_>,
    Runner<'_, &mut WifiDevice<'_, WifiStaDevice>>,
)> = static_cell::StaticCell::new();

static ESP_WIFI_CONTROLLER: static_cell::StaticCell<esp_wifi::EspWifiController<'_>> =
    const { static_cell::StaticCell::new() };
static ESP_WIFI_DEVICE: static_cell::StaticCell<WifiDevice<'_, WifiStaDevice>> =
    const { static_cell::StaticCell::new() };

pub fn get_runner_controller_stack() -> (
    &'static Stack<'static>,
    &'static mut Runner<'static, &'static mut WifiDevice<'static, WifiStaDevice>>,
    WifiController<'static>,
) {
    const MINIMAL_HEAP_REQUIRED: usize = 72 * 1024;
    esp_alloc::heap_allocator!(MINIMAL_HEAP_REQUIRED);

    esp_println::logger::init_logger_from_env();

    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    let wifi_controller_timer = TimerGroup::new(peripherals.TIMG0).timer0;
    let random_number_generator = Rng::new(peripherals.RNG);
    let radio_clock = peripherals.RADIO_CLK;
    let wifi = peripherals.WIFI;
    let timg1_0 = TimerGroup::new(peripherals.TIMG1).timer0;

    esp_hal_embassy::init(timg1_0);

    let (wifi_device_tmp, controller) = match esp_wifi::wifi::new_with_mode(
        ESP_WIFI_CONTROLLER.uninit().write(
            esp_wifi::init(wifi_controller_timer, random_number_generator, radio_clock).unwrap(),
        ),
        wifi,
        WifiStaDevice,
    ) {
        Ok(v) => v,
        Err(e) => panic!("esp wifi new went wrong : {:?}", e),
    };
    let wifi_device = ESP_WIFI_DEVICE.uninit().write(wifi_device_tmp);

    let (stack, runner) = NETWORK_STACK_CELL.uninit().write(new(
        wifi_device,
        embassy_net::Config::dhcpv4(Default::default()),
        NETWORK_STACK_RESSOURCES_CELL
            .uninit()
            .write(StackResources::<3>::new()),
        // TODO : Generate random
        1234,
    ));
    (stack, runner, controller)
}
