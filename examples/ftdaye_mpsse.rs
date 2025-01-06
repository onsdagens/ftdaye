use ftdaye::ftdaye::mpsse::cmd_write_imm;
use ftdaye::ftdaye::{BitMode, Device, Interface};
use log::*;
use nusb::DeviceInfo;
use std::{
    io::{Read, Write},
    time::Duration,
};

fn main() {
    pretty_env_logger::init();

    let device_info = nusb::list_devices()
        .unwrap()
        .find(|dev| dev.vendor_id() == 0x0403 && dev.product_id() == 0x6010)
        .expect("device not connected");

    debug!("device_info {:?}", device_info);

    let mut device = ftdaye::ftdaye::Builder::new()
        .with_interface(Interface::A)
        .with_read_timeout(Duration::from_secs(5))
        .with_write_timeout(Duration::from_secs(5))
        .usb_open(device_info)
        .unwrap();

    device.usb_reset().unwrap();
    // 0x0B configures pins for JTAG
    device.set_bitmode(0x0b, BitMode::Mpsse).unwrap();
    device.set_latency_timer(1).unwrap();
    device.usb_purge_buffers().unwrap();

    let mut junk = vec![];
    let _ = device.read_to_end(&mut junk);

    let (output, direction) = (0x0088, 0x008b);
    debug!(
        "vendor id {:x?}\nproduct id {:x?}, string {:?}",
        device.vendor_id(),
        device.product_id(),
        device.product_string()
    );
    debug!("pinmode {:x} {:x}", output, direction);
    device.set_pins(output, direction).unwrap();

    let speed_khz = 1000;

    // FTDI 2232
    // Disable divide-by-5 mode
    device.disable_divide_by_5().unwrap();
    let buffer_size_bytes: u16 = 4096;
    let max_clock_khz: u32 = 30_000;

    // If `speed_khz` is not a divisor of the maximum supported speed, we need to round up
    let is_exact = max_clock_khz % speed_khz == 0;

    // If `speed_khz` is 0, use the maximum supported speed
    let divisor = (max_clock_khz.checked_div(speed_khz).unwrap_or(1) - is_exact as u32).min(0xFFFF);

    let actual_speed = max_clock_khz / (divisor + 1);

    info!(
        "Setting speed to {} kHz (divisor: {}, actual speed: {} kHz)",
        speed_khz, divisor, actual_speed
    );

    device.configure_clock_divider(divisor as u16).unwrap();

    device.disable_loopback().unwrap();

    // check bac command
    let bad_command = [0xAB];
    device.write_all(&bad_command);

    let mut junk = vec![];
    let r = device.read_to_end(&mut junk);

    debug!("r {:?}, buf {:x?}", r, junk);

    println!("-- reset --");
    reset_to_rti(&mut device);

    println!("read idcode through register");
    rti_to_shift_ir(&mut device);
    shift_ir(&mut device, 0b00_1001);

    rti_to_shift_dr(&mut device);
    device.write(&cmd_write_imm(&[0, 0, 0, 0])).unwrap();
    let mut b4 = [0u8; 4];
    device.read_exact(&mut b4).unwrap();
    println!("read {:#04x?}", b4);
    let idcode = u32::from_le_bytes(b4);
    println!("read {:#010x?}", idcode);
    assert_eq!(idcode, 0x0362d093);
    dr_to_rti(&mut device);

    println!("write user3 through setting ir");
    rti_to_shift_ir(&mut device);
    shift_ir(&mut device, 0b10_0010);

    rti_to_shift_dr(&mut device);
    device
        .write(&cmd_write_imm(&[
            0x1, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        ]))
        .unwrap();
    let mut b8 = [0u8; 8];
    device.read_exact(&mut b8);
    println!("first read {:x?}", b8);
    dr_to_rti(&mut device);

    rti_to_shift_dr(&mut device);
    device
        .write(&cmd_write_imm(&[0xde, 0xad, 0xbe, 0xef, 1, 3, 3, 7]))
        .unwrap();
    let mut b8 = [0u8; 8];
    device.read_exact(&mut b8);
    println!("second read {:x?}", b8);
    dr_to_rti(&mut device);

    rti_to_shift_dr(&mut device);
    device
        .write(&cmd_write_imm(&[0, 0, 0, 0, 0, 0, 0, 0]))
        .unwrap();
    let mut b8 = [0u8; 8];
    device.read_exact(&mut b8);
    println!("3rd read {:x?}", b8);
    dr_to_rti(&mut device);
}

// reset state machine, and go to rti
fn reset_to_rti(device: &mut Device) {
    device.write(&[0x4b, 5, 0b11111, 0x87]).unwrap();
    device.write(&[0x4b, 0, 0b0, 0x87]).unwrap();
}

fn rti_to_shift_dr(device: &mut Device) {
    device.write(&[0x4b, 2, 0b001, 0x87]).unwrap();
}

fn rti_to_shift_ir(device: &mut Device) {
    device.write(&[0x4b, 3, 0b0011, 0x87]).unwrap();
}

fn dr_to_rti(device: &mut Device) {
    device.write(&[0x4b, 2, 0b011, 0x87]).unwrap();
}

fn ir_to_rti(device: &mut Device, bit7: u8) {
    device.write(&[0x4b, 2, bit7 | 0b011, 0x87]).unwrap();
}

fn shift_ir(device: &mut Device, ir: u8) {
    // 5 bits of ir
    device.write(&[0x1b, 4, ir, 0x87]).unwrap();
    // msb of ir as bit 7 of next transaction
    ir_to_rti(device, (ir & 0b10_0000) << 2);
}
