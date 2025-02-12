use ftdaye::{
    ftdaye::{jtag::FtdiMpsse, Interface},
    xilinx7::{IR_IDCODE, IR_USER3, IR_USER4, IR_USERCODE},
};
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
        .with_interface(Interface::A)
        .with_read_timeout(Duration::from_secs(5))
        .with_write_timeout(Duration::from_secs(5))
        .usb_open(device_info)
        .unwrap();

    let mut ft = FtdiMpsse::new(device, 1000);

    println!("-- reset --");
    ft.reset_and_to_rti();

    ft.rti_to_shift_ir();
    ft.shift_ir(IR_USER3);
    //ft.rti_to_shift_ir();
    //ft.reset_and_to_rti();
    ft.shift_ir_bytes(&[3u8, 5u8, 7u8, 9u8]);
    // let mut data = [0u8; 4];
    // ft.read_register(IR_IDCODE, &mut data);
    // println!("read data   {:#04x?}", data);
    // let idcode = u32::from_le_bytes(data);
    // println!("read idcode {:#010x?}", idcode);
    // assert_eq!(idcode, 0x0362d093);

    // ft.read_register(IR_USERCODE, &mut data);
    // println!("read data   {:#04x?}", data);
    // let usercode = u32::from_le_bytes(data);
    // println!("read usercode {:#010x?}", usercode);
    // assert_eq!(usercode, 0x00102030);

    //let mut data = [0u8; 4096 * 4 - 1];
    //ft.read_write_register(IR_USER3, &mut data);
    //println!("Data written");

    //println!("write user3 through setting ir");
    //let data = [0x1, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    //ft.write_register(IR_USER3, &data);
    //println!("first write {:x?}", data);
    //ft.assert_ftdi_buffer_empty();

    //let mut data = [
    //    0xde, 0xad, 0xbe, 0xef, 1, 3, 3, 7, 0xde, 0xad, 0xbe, 0xef, 1, 3, 3, 7, 8,
    //];
    //ft.read_write_register(IR_USER3, &mut data);
    //println!("read write {:x?}", data);

    //let mut data = [0x0; 8];
    //ft.read_register(IR_USER3, &mut data);
    //println!("third read {:x?}", data);

    //let mut data = [0b11000001];
    //ft.read_write_register(IR_USER3, &mut data);
    //println!("1 byte read write {:x?}", data);

    //let mut data = [0x0; 3];
    //ft.read_register(IR_USER3, &mut data);
    //println!("3 byte read {:x?}", data);
}
