#![no_std]
#![no_main]

use {core::panic::PanicInfo, nrf52833_hal as hal, rtt_target::rprintln};

#[rtic::app(device = crate::hal::pac, peripherals = true, dispatchers = [SWI0_EGU0, SWI1_EGU1])]
mod app {
    use cortex_m::asm::nop;
    use embedded_hal::digital::OutputPin;
    use nrf52833_hal::{
        self as hal,
        gpio::{
            p0::{Parts, P0_21},
            Level, Output, PushPull,
        },
    };
    use rtic::Monotonic;
    use rtt_target::{rprintln, rtt_init_print};
    use systick_monotonic::*;

    #[monotonic(binds = SysTick, default = true)]
    type Timer = Systick<1_000_000>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        row1: P0_21<Output<PushPull>>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_init_print!();
        let p0: Parts = hal::gpio::p0::Parts::new(ctx.device.P0);
        let _col1 = p0.p0_28.into_push_pull_output(Level::Low);
        let mut row1: P0_21<Output<PushPull>> = p0.p0_21.into_push_pull_output(Level::Low);
        let _ = row1.set_high();
        let mono = Systick::new(ctx.core.SYST, 64_000_000);
        test_task::spawn().unwrap();
        rprintln!("hello world");
        if test_task::spawn().is_err() {
            rprintln!("error spawning");
        } else {
            rprintln!("spawned");
        }
        (Shared {}, Local { row1 }, init::Monotonics(mono))
    }

    #[task(priority = 1)]
    fn test_task(_ctx: test_task::Context) {
        rprintln!("my test task");
    }

    #[idle(local = [row1])]
    fn idle(ctx: idle::Context) -> ! {
        let row1_pin = ctx.local.row1;
        let _ = row1_pin.set_low();
        rprintln!("idle...");
        loop {
            nop();
        }
    }
}

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cortex_m::interrupt::disable();
    rprintln!("{}", info);
    loop {}
}
