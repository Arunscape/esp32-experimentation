#![no_std]
#![no_main]

use defmt::info;
use esp_hal::clock::CpuClock;
use {defmt_rtt as _, esp_backtrace as _};

use core::ptr::addr_of_mut;
use defmt::println;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use esp_hal::system::Cpu;
use esp_hal::{
    gpio::{Level, Output, OutputConfig},
    interrupt::{software::SoftwareInterruptControl, Priority},
    // system::{CpuControl, Stack, Cpu},
    timer::{timg::TimerGroup, AnyTimer},
};
use esp_hal_embassy::InterruptExecutor;
use static_cell::StaticCell;

use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Ticker, Timer};
use esp_backtrace as _;
// use esp_hal::psram;
use core::mem::MaybeUninit;

extern crate alloc;

// static mut APP_CORE_STACK: Stack<8192> = Stack::new();

// /// Waits for a message that contains a duration, then flashes a led for that
// /// duration of time.
// #[embassy_executor::task]
// async fn control_led(
//     mut led: Output<'static>,
//     control: &'static Signal<CriticalSectionRawMutex, bool>,
// ) {
//     defmt::println!("Starting control_led() on core {}", Cpu::current() as usize);
//     loop {
//         if control.wait().await {
//             defmt::println!("LED on");
//             led.set_low();
//         } else {
//             defmt::println!("LED off");
//             led.set_high();
//         }
//     }
// }

// /// Sends periodic messages to control_led, enabling or disabling it.
// #[embassy_executor::task]
// async fn enable_disable_led(control: &'static Signal<CriticalSectionRawMutex, bool>) {
//     defmt::println!(
//         "Starting enable_disable_led() on core {}",
//         Cpu::current() as usize
//     );
//     let mut ticker = Ticker::every(Duration::from_secs(1));
//     loop {
//         defmt::println!("Sending LED on");
//         control.signal(true);
//         ticker.next().await;

//         defmt::println!("Sending LED off");
//         control.signal(false);
//         ticker.next().await;
//     }
// }

// #[esp_hal_embassy::main]
// async fn main(spawner: Spawner) {
//     // generator version: 0.2.2

//     // let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
//     let config = esp_hal::Config::default();
//     let peripherals = esp_hal::init(config);

//     // esp_alloc::heap_allocator!(72 * 1024);

//     // let timer0 = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER);
//     // esp_hal_embassy::init(timer0.alarm0);
//     let sw_ints = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
//     let timg0 = TimerGroup::new(peripherals.TIMG0);
//     let timer0: AnyTimer = timg0.timer0.into();
//     let timer1: AnyTimer = timg0.timer1.into();
//     esp_hal_embassy::init([timer0, timer1]);

//     info!("Embassy initialized!");

//     // let timer1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
//     // let _init = esp_wifi::init(
//     //     timer1.timer0,
//     //     esp_hal::rng::Rng::new(peripherals.RNG),
//     //     peripherals.RADIO_CLK,
//     // )
//     // .unwrap();

//     // // TODO: Spawn some tasks
//     // let _ = spawner;

//     // loop {
//     //     info!("Hello world!");
//     //     Timer::after(Duration::from_secs(1)).await;
//     // }

//     // // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/v0.23.1/examples/src/bin

//     let mut cpu_control = CpuControl::new(peripherals.CPU_CTRL);

//     static LED_CTRL: StaticCell<Signal<CriticalSectionRawMutex, bool>> = StaticCell::new();
//     let led_ctrl_signal = &*LED_CTRL.init(Signal::new());

//     let led = Output::new(peripherals.GPIO0, Level::Low, OutputConfig::default());

//     static EXECUTOR_CORE_1: StaticCell<InterruptExecutor<1>> = StaticCell::new();
//     let executor_core1 = InterruptExecutor::new(sw_ints.software_interrupt1);
//     let executor_core1 = EXECUTOR_CORE_1.init(executor_core1);

//     let _guard = cpu_control
//         .start_app_core(unsafe { &mut *addr_of_mut!(APP_CORE_STACK) }, move || {
//             let spawner = executor_core1.start(Priority::Priority1);

//             spawner.spawn(control_led(led, led_ctrl_signal)).ok();

//             // Just loop to show that the main thread does not need to poll the executor.
//             loop {}
//         })
//         .unwrap();

//     static EXECUTOR_CORE_0: StaticCell<InterruptExecutor<0>> = StaticCell::new();
//     let executor_core0 = InterruptExecutor::new(sw_ints.software_interrupt0);
//     let executor_core0 = EXECUTOR_CORE_0.init(executor_core0);

//     let spawner = executor_core0.start(Priority::Priority1);
//     spawner.spawn(enable_disable_led(led_ctrl_signal)).ok();

//     // Just loop to show that the main thread does not need to poll the executor.
//     loop {}
// }

/// Periodically print something.
#[embassy_executor::task]
async fn high_prio() {
    println!("Starting high_prio()");
    let mut ticker = Ticker::every(Duration::from_secs(1));
    loop {
        println!("High priority ticks");
        ticker.next().await;
    }
}

/// Simulates some blocking (badly behaving) task.
#[embassy_executor::task]
async fn low_prio_blocking() {
    println!("Starting low-priority task that isn't actually async");
    loop {
        println!("Doing some long and complicated calculation");
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(5) {}
        println!("Calculation finished");
        Timer::after(Duration::from_secs(5)).await;
    }
}

/// A well-behaved, but starved async task.
#[embassy_executor::task]
async fn low_prio_async() {
    println!(
        "Starting low-priority task that will not be able to run while the blocking task is running"
    );
    let mut ticker = Ticker::every(Duration::from_secs(1));
    loop {
        println!("Low priority ticks");
        ticker.next().await;
    }
}

// #[global_allocator]
// static PSRAM_ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

// fn init_psram_heap() {
//     unsafe {
//         PSRAM_ALLOCATOR.add_region(esp_alloc::HeapRegion::new(
//             psram::psram_vaddr_start() as *mut u8,
//             psram::PSRAM_BYTES,
//             esp_alloc::MemoryCapability::Internal.into(),
//         ));
//     }
// }

#[esp_hal_embassy::main]
async fn main(low_prio_spawner: Spawner) {
    // init_psram_heap();

    const HEAP_SIZE: usize = 32 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        esp_alloc::HEAP.add_region(esp_alloc::HeapRegion::new(
            HEAP.as_mut_ptr() as *mut u8,
            HEAP_SIZE,
            esp_alloc::MemoryCapability::Internal.into(),
        ));
    }


    let peripherals = esp_hal::init(esp_hal::Config::default());

    let sw_ints = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let timer0: AnyTimer = timg0.timer0.into();

    cfg_if::cfg_if! {
        if #[cfg(feature = "esp32c2")] {
            use esp_hal::timer::systimer::SystemTimer;
            let systimer = SystemTimer::new(peripherals.SYSTIMER);
            let timer1: AnyTimer = systimer.alarm0.into();
        } else {
            let timg1 = TimerGroup::new(peripherals.TIMG1);
            let timer1: AnyTimer = timg1.timer0.into();
        }
    }

    esp_hal_embassy::init([timer0, timer1]);

    static EXECUTOR: StaticCell<InterruptExecutor<2>> = StaticCell::new();
    let executor = InterruptExecutor::new(sw_ints.software_interrupt2);
    let executor = EXECUTOR.init(executor);

    let spawner = executor.start(Priority::Priority3);
    spawner.must_spawn(high_prio());

    println!("Spawning low-priority tasks");
    low_prio_spawner.must_spawn(low_prio_async());
    low_prio_spawner.must_spawn(low_prio_blocking());
    loop {}
}
