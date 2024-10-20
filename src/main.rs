#![no_std]
#![no_main]

mod monotonic_nrf52;

use embedded_hal::digital::{OutputPin, StatefulOutputPin};
use fugit::{self, ExtU32};
use nrf52833_hal::gpio::{
    p0::{P0_15, P0_19, P0_21, P0_22, P0_24},
    Level, Output, PushPull,
};
use {core::panic::PanicInfo, nrf52833_hal as hal, rtt_target::rprintln};

pub struct LedRows {
    led1: P0_21<Output<PushPull>>,
    led2: P0_22<Output<PushPull>>,
    led3: P0_15<Output<PushPull>>,
    led4: P0_24<Output<PushPull>>,
    led5: P0_19<Output<PushPull>>,
}

impl LedRows {
    pub fn is_all_turned_on(&mut self) -> bool {
        self.led1.is_set_high().unwrap()
            && self.led2.is_set_high().unwrap()
            && self.led3.is_set_high().unwrap()
            && self.led4.is_set_high().unwrap()
            && self.led5.is_set_high().unwrap()
    }

    pub fn turn_on_all(&mut self) {
        let _ = self.led1.set_high();
        let _ = self.led2.set_high();
        let _ = self.led3.set_high();
        let _ = self.led4.set_high();
        let _ = self.led5.set_high();
    }

    pub fn turn_off_all(&mut self) {
        let _ = self.led1.set_low();
        let _ = self.led2.set_low();
        let _ = self.led3.set_low();
        let _ = self.led4.set_low();
        let _ = self.led5.set_low();
    }
}

#[rtic::app(device = crate::hal::pac, peripherals = true, dispatchers = [SWI0_EGU0])]
mod app {
    use super::*;

    use cortex_m::asm::nop;
    use embedded_hal::digital::InputPin;
    use monotonic_nrf52::MonoTimer;
    use nrf52833_hal::{
        self as hal,
        gpio::{p0::Parts, Input, Pin, PullUp},
        gpiote::Gpiote,
    };
    use rtic::Monotonic;
    use rtt_target::{rprintln, rtt_init_print};

    #[monotonic(binds = TIMER1, default = true)]
    type Tonic = MonoTimer<nrf52833_hal::pac::TIMER1>;

    #[shared]
    struct Shared {
        gpiote: Gpiote,
        led_rows: LedRows,
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
        let led_rows = LedRows {
            led1: p0.p0_21.into_push_pull_output(Level::Low),
            led2: p0.p0_22.into_push_pull_output(Level::Low),
            led3: p0.p0_15.into_push_pull_output(Level::Low),
            led4: p0.p0_24.into_push_pull_output(Level::Low),
            led5: p0.p0_19.into_push_pull_output(Level::Low),
        };
        let btn1 = p0.p0_14.into_pullup_input().degrade();
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
            Shared { gpiote, led_rows },
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

    #[task(shared = [gpiote, led_rows], local = [btn1])]
    fn debounce(mut ctx: debounce::Context) {
        let btn1_pressed = ctx.local.btn1.is_low().unwrap();
        let led_is_light = ctx
            .shared
            .led_rows
            .lock(|led_rows| led_rows.is_all_turned_on());
        if btn1_pressed {
            ctx.shared.led_rows.lock(|led_rows| {
                if led_is_light {
                    led_rows.turn_off_all();
                } else {
                    led_rows.turn_on_all();
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
