#![no_std]
#![no_main]

mod monotonic_nrf52;

use fugit::{self, ExtU32};
use {core::panic::PanicInfo, nrf52833_hal as hal, rtt_target::rprintln};

#[rtic::app(device = crate::hal::pac, peripherals = true, dispatchers = [SWI0_EGU0])]
mod app {
    use super::*;

    use cortex_m::asm::nop;
    use embedded_hal::digital::{InputPin, OutputPin, StatefulOutputPin};
    use monotonic_nrf52::MonoTimer;
    use nrf52833_hal::{
        self as hal,
        gpio::{
            p0::{Parts, P0_21},
            Input, Level, Output, Pin, PullUp, PushPull,
        },
        gpiote::Gpiote,
    };
    use rtic::Monotonic;
    use rtt_target::{rprintln, rtt_init_print};

    #[monotonic(binds = TIMER1, default = true)]
    type Tonic = MonoTimer<nrf52833_hal::pac::TIMER1>;

    #[shared]
    struct Shared {
        gpiote: Gpiote,
        led1: P0_21<Output<PushPull>>,
    }

    #[local]
    struct Local {
        btn1: Pin<Input<PullUp>>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_init_print!();
        let p0: Parts = hal::gpio::p0::Parts::new(ctx.device.P0);
        let _col1 = p0.p0_28.into_push_pull_output(Level::Low);
        let mut row1: P0_21<Output<PushPull>> = p0.p0_21.into_push_pull_output(Level::Low);
        let btn1 = p0.p0_14.into_pullup_input().degrade();
        let _ = row1.set_high();
        let mono = MonoTimer::new(ctx.device.TIMER1);

        let gpiote = Gpiote::new(ctx.device.GPIOTE);
        gpiote
            .channel0()
            .input_pin(&btn1)
            .hi_to_lo()
            .enable_interrupt();
        foo::spawn().ok();
        rprintln!("init finished");
        (
            Shared { gpiote, led1: row1 },
            Local { btn1 },
            init::Monotonics(mono),
        )
    }

    #[task]
    fn foo(_: foo::Context) {
        // rprintln!("foo");
        foo::spawn_after(2000.millis()).ok();
    }

    #[task(binds = GPIOTE, shared = [gpiote])]
    fn on_gpiote(mut ctx: on_gpiote::Context) {
        ctx.shared.gpiote.lock(|gpiote| {
            rprintln!("on_gpiote");
            gpiote.reset_events();
            debounce::spawn_after(50.millis()).ok();
        });
    }

    #[task(shared = [gpiote, led1], local = [btn1])]
    fn debounce(mut ctx: debounce::Context) {
        let btn1_pressed = ctx.local.btn1.is_low().unwrap();
        let led_is_light = ctx.shared.led1.lock(|led1| led1.is_set_low().unwrap());
        if btn1_pressed {
            ctx.shared.led1.lock(|led1| {
                if led_is_light {
                    let _ = led1.set_high();
                } else {
                    let _ = led1.set_low();
                }
            });
        }
    }

    #[idle()]
    fn idle(_: idle::Context) -> ! {
        loop {
            rprintln!("idle...");
            for _ in 0..300_000 {
                nop();
            }
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
