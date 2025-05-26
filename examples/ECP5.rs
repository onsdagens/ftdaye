use ftdaye::ftdaye::{jtag::FtdiMpsse, Interface};
use log::*;

use std::time::Duration;

fn main() {
    pretty_env_logger::init();

    let device_info = nusb::list_devices()
        .unwrap()
        .find(|dev| dev.vendor_id() == 0x0403 && dev.product_id() == 0x6010)
        .expect("device not connected");

    debug!("device_info {:?}", device_info);

    let device = ftdaye::ftdaye::Builder::new()
        .with_interface(Interface::B)
        .with_read_timeout(Duration::from_secs(5))
        .with_write_timeout(Duration::from_secs(5))
        .usb_open(device_info)
        .unwrap();

    let mut ft = FtdiMpsse::new(device, 1000);

    println!("-- reset --");
    ft.reset_and_to_rti();
    let mut data = [0 as u8; 4];
    // the lib side implementation expects IR to be 5 bits wide, ours is only 4
    // so concat something to right side (it will be shifted out by the 5th shift)
    ft.read_register(0b01100, &mut data);
    println!("IDCODE: 0x{:08X}", u32::from_le_bytes(data));
    assert_eq!(data, [0xef, 0xbe, 0xad, 0xde]);

    // now, write some data to reg 0x1.
    let mut data = [0x0A as u8; 1];
    // again the actual address is 0b0001, but the address is expected to be 5 bits wide by the
    // lib.
    ft.write_register(0b00010, &mut data);
}
