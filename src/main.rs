#![no_std]
#![no_main]

use cortex_m::asm::nop;
use cortex_m_rt::entry;
use embedded_hal::digital::{InputPin, OutputPin, PinState};
use hal::pac;
use nrf52833_hal::{
    self as hal,
    gpio::{
        p0::{P0_21, P0_28},
        Level, Output, PushPull,
    },
    pac::Peripherals,
};
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let p: Peripherals = pac::Peripherals::take().unwrap();
    let port0 = hal::gpio::p0::Parts::new(p.P0);

    let _col1: P0_28<Output<PushPull>> = port0.p0_28.into_push_pull_output(Level::Low);
    let mut row1: P0_21<Output<PushPull>> = port0.p0_21.into_push_pull_output(Level::Low);

    let mut btn_a = port0.p0_14.into_pullup_input();

    rprintln!("Hello world");
    loop {
        let btn_state = btn_a.is_low().unwrap();
        rprintln!("btn_a is: {}", btn_state);
        let _ = row1.set_state(PinState::from(btn_state));
        for _ in 0..100_000 {
            nop();
        }
    }
}
