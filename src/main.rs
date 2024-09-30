#![no_std]
#![no_main]

use {core::panic::PanicInfo, nrf52833_hal as hal, rtt_target::rprintln};

#[rtic::app(device = crate::hal::pac, peripherals = true, dispatchers = [SWI0_EGU0, SWI1_EGU1])]
mod app {
    use cortex_m::asm::nop;
    use embedded_hal::digital::{InputPin, OutputPin};
    use fugit::Instant;
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
    use systick_monotonic::*;

    #[monotonic(binds = TIMER0, default = true)]
    type Timer = Systick<1_000_000>;

    #[shared]
    struct Shared {
        gpiote: Gpiote,
    }

    #[local]
    struct Local {
        row1: P0_21<Output<PushPull>>,
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
        let mono = Systick::new(ctx.core.SYST, 64_000_000);

        let gpiote = Gpiote::new(ctx.device.GPIOTE);
        gpiote
            .channel0()
            .input_pin(&btn1)
            .hi_to_lo()
            .enable_interrupt();
        rprintln!("init finished");
        (
            Shared { gpiote },
            Local { row1, btn1 },
            init::Monotonics(mono),
        )
    }

    #[task(binds = GPIOTE, shared = [gpiote])]
    fn on_gpiote(mut ctx: on_gpiote::Context) {
        // let delay = 50.millis();
        ctx.shared.gpiote.lock(|gpiote| {
            rprintln!("test");
            gpiote.reset_events();
        });
    }

    #[task(shared = [gpiote], local = [btn1], priority = 1)]
    fn debounce(mut ctx: debounce::Context) {
        let btn1_pressed = ctx.local.btn1.is_low().unwrap();
        rprintln!("button 1: {}", btn1_pressed);
        ctx.shared.gpiote.lock(|gpiote| {
            if btn1_pressed {
                rprintln!("Button 1 was pressed!");
                // Manually run "task out" operation (toggle) on channel 1 (toggles led1)
                gpiote.channel0().out();
            }
        });
    }

    #[idle(local = [row1])]
    fn idle(ctx: idle::Context) -> ! {
        let row1_pin = ctx.local.row1;
        let _ = row1_pin.set_low();
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
