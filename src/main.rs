#![no_std]
#![no_main]

use cortex_m_rt::entry;

use defmt_rtt as _;
use panic_probe as _;

use embedded_time::fixed_point::FixedPoint;

use rp_pico::hal::{
    pac::{self, interrupt},
    prelude::*,
    self,
};

use usb_device::{class_prelude::*, prelude::*};
use usbd_hid::descriptor::generator_prelude::*;
use usbd_hid::hid_class::HIDClass;

use pedals::{PedalReport, Pedals};

mod pedals;


static mut USB_DEVICE: Option<UsbDevice<hal::usb::UsbBus>> = None;
static mut USB_BUS: Option<UsbBusAllocator<hal::usb::UsbBus>> = None;
static mut USB_HID: Option<HIDClass<hal::usb::UsbBus>> = None;

const POLL_MS: u8 = 10;
const REPORT_UPDATE_MS: u32 = 5;

/// Entry point to our bare-metal application.
///
/// The `#[entry]` macro ensures the Cortex-M start-up code calls this function
/// as soon as all global variables are initialised.
#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let usb_bus = hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    );

    let usb_bus_alloc = UsbBusAllocator::new(usb_bus);

    unsafe {
        USB_BUS = Some(usb_bus_alloc);
        
        let bus_ref = USB_BUS.as_ref()
            .unwrap();

        let usb_hid = HIDClass::new(
            bus_ref,
            PedalReport::desc(),
            POLL_MS,
        );

        // TODO: There's probably a legitimate VID/PID pair we could use
        // but fuggit
        let usb_dev = UsbDeviceBuilder::new(bus_ref, UsbVidPid(0x27dc, 0x16c0))
            .manufacturer("Yeebs Industries")
            .product("Junk Pedals")
            .device_class(0xEF) // misc
            .build();

        USB_HID = Some(usb_hid);
        USB_DEVICE = Some(usb_dev);
    }

    let sio = hal::Sio::new(pac.SIO);
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Enable ADC
    let adc = hal::Adc::new(pac.ADC, &mut pac.RESETS);

    let adc2_pin = pins.gpio28.into_floating_input();
    let adc1_pin = pins.gpio27.into_floating_input();

    // Enable the USB interrupt
    unsafe {
        pac::NVIC::unmask(hal::pac::Interrupt::USBCTRL_IRQ);
    };

    let core = pac::CorePeripherals::take().unwrap();
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer());

    let mut pedals = Pedals::new(adc, adc2_pin, adc1_pin);

    loop {
        let report = pedals.report();

        gamepad_update(report)
            .ok()
            .unwrap_or(0);

        delay.delay_ms(REPORT_UPDATE_MS);
        //defmt::info!("report: {}, {}", report.x, report.y);
    }
}

fn gamepad_update(report: PedalReport) -> Result<usize, usb_device::UsbError> {
    cortex_m::interrupt::free(|_| unsafe {
        // Now interrupts are disabled, grab the global variable and, if
        // available, send it a HID report
        USB_HID.as_mut().map(|hid| hid.push_input(&report))
    })
    .unwrap()
}


#[allow(non_snake_case)]
#[interrupt]
unsafe fn USBCTRL_IRQ() {
    let usb_dev = USB_DEVICE.as_mut().unwrap();
    let usb_hid = USB_HID.as_mut().unwrap();

    usb_dev.poll(&mut [usb_hid]);
}
