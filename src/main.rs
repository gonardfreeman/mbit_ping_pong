#![no_std]
#![no_main]

use {core::panic::PanicInfo, rtt_target::rprintln};

#[rtic::app(device = hal::pac, peripherals = true, dispatchers = [SWI0_EGU0])]
mod app {
    use embedded_hal::digital::InputPin;
    use systick_monotonic::*;

    use {
        hal::{
            gpio::{Input, Pin, PullUp},
            gpiote::*,
        },
        nrf52833_hal as hal,
        rtt_target::{rprintln, rtt_init_print},
    };

    #[monotonic(binds = SysTick, default = true)]
    type Timer = Systick<1_000_000>;

    #[shared]
    struct Shared {
        gpiote: Gpiote,
    }

    #[local]
    struct Local {
        btn_a: Pin<Input<PullUp>>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let _clocks = hal::clocks::Clocks::new(ctx.device.CLOCK).enable_ext_hfosc();
        rtt_init_print!();
        let p0 = hal::gpio::p0::Parts::new(ctx.device.P0);
        let btn_a = p0.p0_14.into_pullup_input().degrade();

        let gpiote = Gpiote::new(ctx.device.GPIOTE);

        gpiote
            .channel0()
            .input_pin(&btn_a)
            .hi_to_lo()
            .enable_interrupt();

        gpiote.port().enable_interrupt();

        let mono = Systick::new(ctx.core.SYST, 64_000_000);
        rprintln!("Press a button");

        (Shared { gpiote }, Local { btn_a }, init::Monotonics(mono))
    }

    #[task(binds = GPIOTE, shared = [gpiote])]
    fn on_gpiote(mut ctx: on_gpiote::Context) {
        rprintln!("on gpiote");
        ctx.shared.gpiote.lock(|gpiote| {
            if gpiote.channel0().is_event_triggered() {
                rprintln!("Interrupt from channel 0 event");
            }
            if gpiote.port().is_event_triggered() {
                rprintln!("Interrupt from port event");
            }
            // Reset all events
            gpiote.reset_events();
            // Debounce
            debounce::spawn_after(50.millis()).ok();
        });
    }

    #[task(shared = [gpiote], local = [btn_a])]
    fn debounce(mut ctx: debounce::Context) {
        rprintln!("on debounce");
        let btn_a_pressed = ctx.local.btn_a.is_low().unwrap();

        ctx.shared.gpiote.lock(|gpiote| {
            if btn_a_pressed {
                rprintln!("Button 1 was pressed!");
                gpiote.channel1().out();
            }
        });
    }
}

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cortex_m::interrupt::disable();
    rprintln!("{}", info);
    loop {}
}
