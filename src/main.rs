#![no_std]
#![no_main]

use cortex_m::asm::nop;
use cortex_m_rt::entry;
use embedded_hal::digital::{/*InputPin,*/ OutputPin /*, PinState*/};
use hal::pac;
use nrf52833_hal::{
    self as hal,
    gpio::{
        p0::{P0_15, P0_19, P0_21, P0_22, P0_24, P0_28},
        Level, Output, PushPull,
    },
    pac::Peripherals,
};
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};

enum GpioPin {
    Pin1(P0_21<Output<PushPull>>),
    Pin2(P0_22<Output<PushPull>>),
    Pin3(P0_15<Output<PushPull>>),
    Pin4(P0_24<Output<PushPull>>),
    Pin5(P0_19<Output<PushPull>>),
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let p: Peripherals = pac::Peripherals::take().unwrap();
    let port0 = hal::gpio::p0::Parts::new(p.P0);
    let mut pins: [GpioPin; 5] = [
        GpioPin::Pin1(port0.p0_21.into_push_pull_output(Level::Low)),
        GpioPin::Pin2(port0.p0_22.into_push_pull_output(Level::Low)),
        GpioPin::Pin3(port0.p0_15.into_push_pull_output(Level::Low)),
        GpioPin::Pin4(port0.p0_24.into_push_pull_output(Level::Low)),
        GpioPin::Pin5(port0.p0_19.into_push_pull_output(Level::Low)),
    ];
    let _col1: P0_28<Output<PushPull>> = port0.p0_28.into_push_pull_output(Level::Low);

    let _ = port0.p0_14.into_pullup_input();

    rprintln!("Hello world");
    let mut index: usize = 0;
    loop {
        move_left_col_led(index, &mut pins);
        index = (index + 1) % pins.len();
        // let btn_state = btn_a.is_low().unwrap();
        // rprintln!("btn_a is: {}", btn_state);
        // let _ = row1.set_state(PinState::from(btn_state));
        for _ in 0..300_000 {
            nop();
        }
    }
}

fn move_left_col_led(index: usize, pins: &mut [GpioPin; 5]) {
    for (i, pin) in pins.iter_mut().enumerate() {
        if i == index {
            match pin {
                GpioPin::Pin1(p) => p.set_high().unwrap(),
                GpioPin::Pin2(p) => p.set_high().unwrap(),
                GpioPin::Pin3(p) => p.set_high().unwrap(),
                GpioPin::Pin4(p) => p.set_high().unwrap(),
                GpioPin::Pin5(p) => p.set_high().unwrap(),
            }
        } else {
            match pin {
                GpioPin::Pin1(p) => p.set_low().unwrap(),
                GpioPin::Pin2(p) => p.set_low().unwrap(),
                GpioPin::Pin3(p) => p.set_low().unwrap(),
                GpioPin::Pin4(p) => p.set_low().unwrap(),
                GpioPin::Pin5(p) => p.set_low().unwrap(),
            }
        }
    }
}
