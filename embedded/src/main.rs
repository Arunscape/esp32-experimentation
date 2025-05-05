use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::OutputPin;
use esp_idf_svc::hal::gpio::PinDriver; // Although we pass the pin directly to RMT driver
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::rmt::RMT;
use esp_idf_svc::hal::rmt::config::TransmitConfig;
use esp_idf_svc::hal::rmt::{CHANNEL0, FixedLengthSignal, PinState, Pulse, RmtChannel};
use esp_idf_svc::hal::task::block_on;
use esp_idf_svc::hal::units::Hertz;
use esp_idf_svc::log::EspLogger;
use smart_leds::{
    RGB8, SmartLedsWrite,
    hsv::{Hsv, hsv2rgb},
};

// Smart-LEDs related imports
use smart_leds::brightness;
use smart_leds::gamma;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    block_on(async_main())
}
use embassy_time::{Duration, Timer};

async fn async_main() -> anyhow::Result<()> {
    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;

    // Configure RMT and SmartLedsAdapter
    let rmt = peripherals.rmt;

    // Use the specific GPIO pin 8
    let led_pin = pins.gpio8;
    let channel = peripherals.rmt.channel0;

    let config = TransmitConfig::new().clock_divider(1);
    let mut tx_rmt_driver = TxRmtDriver::new(channel, led_pin, &config)?;
    const NUM_LEDS: usize = 1;
    let mut led_buffer = [RGB8::default(); NUM_LEDS]; // Buffer to hold color data
    let mut hue = 0_u8; // Variable for cycling colors

    log::info!("Start NeoPixel rainbow!");

    loop {
        // Generate a simple rainbow pattern
        for i in 0..NUM_LEDS {
            // Calculate hue for this LED
            let current_hue = hue.wrapping_add((i * 256 / NUM_LEDS) as u8);
            // Convert Hsv to Rgb
            led_buffer[i] = hsv2rgb(Hsv {
                hue: current_hue,
                sat: 255, // Full saturation
                val: 30,  // Moderate brightness (adjust as needed)
            });
        }

        // --- Write data to the LED strip ---
        // The TxRmtDriver implements SmartLedsWrite.
        tx_rmt_driver.write(gamma(brightness(led_buffer.iter().cloned(), 50)))?; // Apply gamma and limit brightness to 50/255

        // Increment hue for the next frame
        hue = hue.wrapping_add(4); // Change color faster/slower by adjusting this value

        // Delay between updates (using FreeRtos from esp_idf_svc::hal::delay)
        FreeRtos::delay_ms(50); // ~20 FPS update rate
    }

    // Keep the main task alive (or have other tasks running)
    loop {}
}

//use esp_idf_svc::hal::delay::FreeRtos;
//fn main() -> anyhow::Result<()> {
//    esp_idf_svc::sys::link_patches();
//
//    let peripherals = Peripherals::take()?;
//    let mut led = PinDriver::output(peripherals.pins.gpio8)?;
//
//    loop {
//        led.set_high()?;
//        // we are sleeping here to make sure the watchdog isn't triggered
//        FreeRtos::delay_ms(1000);
//
//        led.set_low()?;
//        FreeRtos::delay_ms(1000);
//    }
//}
