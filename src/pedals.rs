use embedded_hal::adc::{OneShot, Channel};
use rp_pico::hal::{
    self,
    gpio::{Floating, Input, Pin, PinId},
    Adc,
};
use usbd_hid::descriptor::{gen_hid_descriptor, AsInputReport, SerializedDescriptor};
use serde::ser::{Serialize, SerializeTuple, Serializer};


const READ_V_COUNT: usize = 3;

pub type PedalPin<I> = Pin<I, Input<Floating>>;


#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = GAMEPAD) = {
        (collection = PHYSICAL,) = {
            (usage_page = GENERIC_DESKTOP,) = {
                (usage = X,) = {
                    x=input;
                };
                (usage = Y,) = {
                    y=input;
                };
            };
        };
    }
)]
#[allow(dead_code)]
pub struct PedalReport {
    pub x: u8,
    pub y: u8,
}

pub struct Pedals<I: PinId, G: PinId> {
    adc: hal::Adc,
    accel: PedalPin<I>,
    brake: PedalPin<G>,
}


impl<I: PinId, G: PinId> Pedals<I, G>
where
    Pin<I, Input<Floating>>: embedded_hal::adc::Channel<Adc, ID = u8>,
    Pin<G, Input<Floating>>: embedded_hal::adc::Channel<Adc, ID = u8>,
{
    pub fn new(adc: Adc, accel: PedalPin<I>, brake: PedalPin<G>) -> Self {
        Self {
            adc,
            accel,
            brake
        }
    }

    pub fn report(&mut self) -> PedalReport {
        let accel_avg_v = Self::read_average_v(&mut self.adc, &mut self.accel);
        let brake_avg_v = Self::read_average_v(&mut self.adc, &mut self.brake);

        PedalReport {
            x: scale_down_accel(accel_avg_v),
            y: scale_down_brake(brake_avg_v)
        }
    }

    fn read_average_v(adc: &mut Adc, pin: &mut impl Channel<Adc, ID = u8>) -> u16 {
        let mut read_v: [u16; READ_V_COUNT] = [0; READ_V_COUNT];

        for i in 0..READ_V_COUNT {
            read_v[i] = adc.read(pin)
                .unwrap(); // infallible
        }

        let sum: u16 = read_v.iter()
            .sum();

        sum / read_v.len() as u16
    }
}


fn scale_down_brake(v: u16) -> u8 {
    if v <= 750 {
        0
    } else {
        ((v as f32 - 750.0) / 10.9) as u8
    }
}


fn scale_down_accel(v: u16) -> u8 {
    let s = if v <= 650 {
        0
    } else {
        ((v as f32 - 650 as f32) / 10.39) as u8
    };

    255 - s
}
