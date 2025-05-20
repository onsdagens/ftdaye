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

    ft.rti_to_shift_dr();
    ft.reset_and_to_rti();
    ft.rti_to_shift_ir();
}
