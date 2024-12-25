use anyhow::Result;
use bitvec::field::BitField;
use log::*;
use std::{
    cell::Ref,
    convert::TryInto,
    io::{Read, Write},
    thread, time,
};

use ftdaye::{FtdiError, JtagAdapter, FTDI_COMPAT_DEVICES};
fn main() -> Result<()> {
    pretty_env_logger::init();

    let device_info = nusb::list_devices()
        .unwrap()
        .find(|dev| dev.vendor_id() == 0x0403 && dev.product_id() == 0x6010)
        .expect("device not connected");

    let mut jtag_adapter = JtagAdapter::open(FTDI_COMPAT_DEVICES[0], device_info)?;
    debug!("open {:?}", jtag_adapter);

    jtag_adapter.attach()?;
    debug!("attach {:?}", jtag_adapter);

    let bad_command = [0xAB];

    jtag_adapter
        .device
        .write_all(&bad_command)
        .map_err(FtdiError::from)?;

    let mut response = vec![];

    jtag_adapter
        .device
        .read_to_end(&mut response)
        .map_err(FtdiError::from)?;

    debug!("response {:x?}", response);
    assert_eq!(response, &[0xfa, 0xab]);

    // tlr, reset state machine
    let tlr_command = [0x4b, 5, 0b11111, 0x87];
    jtag_adapter
        .device
        .write_all(&tlr_command)
        .map_err(FtdiError::from)?;

    // to shift dr,
    let to_shift_dr_command = [0x4b, 3, 0b0010, 0x87];
    jtag_adapter
        .device
        .write_all(&to_shift_dr_command)
        .map_err(FtdiError::from)?;

    let mut shift_dr_command = vec![];
    for _ in 0..8 {
        // shift_dr
        shift_dr_command.extend_from_slice(&[0x6b, 3, 0b0000]);
    }
    shift_dr_command.push(0x87);

    jtag_adapter
        .device
        .write_all(&shift_dr_command)
        .map_err(FtdiError::from)?;

    let mut response: Vec<u8> = vec![];
    jtag_adapter
        .device
        .read_to_end(&mut response)
        .map_err(FtdiError::from)?;

    debug!("resp {:02x?}", response);

    let idcode = response
        .iter()
        .rev()
        .fold(0, |acc, d| acc << 4 | (*d >> 4) as u32);

    println!("idcode {:#010x}", idcode);
    assert_eq!(idcode, 0x0362d093);

    // // to ir
    // let shift_dr_to_shift_ir_command = [0x4b, 5, 0b001111, 0x87];
    // jtag_adapter
    //     .device
    //     .write_all(&shift_dr_to_shift_ir_command)
    //     .map_err(FtdiError::from)?;

    // // usercode 0b001000, 00102030
    // let shift_ir_command = vec![0x1b, 4, 0b001000, 0x87];
    // jtag_adapter
    //     .device
    //     .write_all(&shift_ir_command)
    //     .map_err(FtdiError::from)?;

    // // to dr
    // let shift_dr_to_shift_ir_command = [0x4b, 4, 0b00111, 0x87];
    // jtag_adapter
    //     .device
    //     .write_all(&shift_dr_to_shift_ir_command)
    //     .map_err(FtdiError::from)?;

    // debug!("--- sleep ---");
    // let ten_millis = time::Duration::from_millis(10);
    // thread::sleep(ten_millis);
    // debug!("--- wake ---");

    // let mut shift_dr_command = vec![];
    // for _ in 0..8 {
    //     // shift_dr
    //     shift_dr_command.extend_from_slice(&[0x6b, 3, 0b0000]);
    // }
    // shift_dr_command.push(0x87);

    // jtag_adapter
    //     .device
    //     .write_all(&shift_dr_command)
    //     .map_err(FtdiError::from)?;

    // let mut response: Vec<u8> = vec![];
    // jtag_adapter
    //     .device
    //     .read_to_end(&mut response)
    //     .map_err(FtdiError::from)?;

    //     debug!("resp {:02x?}", response);

    //     let usercode = response
    //         .iter()
    //         .rev()
    //         .fold(0, |acc, d| acc << 4 | (*d >> 4) as u32);

    //     println!("usercode {:#010x}", usercode);
    //     assert_eq!(usercode, 0x00102030);

    // to rti, and spin there for a while
    let shift_dr_to_rti = [0x4b, 5, 0b000011, 0x87];
    // let shift_dr_to_shift_ir_command = [0x4b, 4, 0b00110, 0x87];
    jtag_adapter
        .device
        .write_all(&shift_dr_to_rti)
        .map_err(FtdiError::from)?;

    // to shift ir
    let shift_dr_to_rti = [0x4b, 3, 0b0011, 0x87];
    // let shift_dr_to_shift_ir_command = [0x4b, 4, 0b00110, 0x87];
    jtag_adapter
        .device
        .write_all(&shift_dr_to_rti)
        .map_err(FtdiError::from)?;

    // user1 0b000010, 12345678
    let shift_ir_command = vec![0x1b, 5, 0b000010, 0x87];
    jtag_adapter
        .device
        .write_all(&shift_ir_command)
        .map_err(FtdiError::from)?;

    // to dr, part 1
    let shift_dr_to_shift_ir_command = [0x4b, 1, 0b11, 0x87];
    jtag_adapter
        .device
        .write_all(&shift_dr_to_shift_ir_command)
        .map_err(FtdiError::from)?;

    // debug!("--- sleep ---");
    // let ten_millis = time::Duration::from_millis(10);
    // thread::sleep(ten_millis);
    // debug!("--- wake ---");

    // to rti, part 2
    let to_rti = [0x4b, 5, 0b0000, 0x87];
    jtag_adapter
        .device
        .write_all(&to_rti)
        .map_err(FtdiError::from)?;

    // to shift_dr part 3
    let to_shift_dr = [0x4b, 2, 0b001, 0x87];
    jtag_adapter
        .device
        .write_all(&to_shift_dr)
        .map_err(FtdiError::from)?;

    let mut shift_dr_command = vec![];
    for _ in 0..8 {
        // shift_dr
        shift_dr_command.extend_from_slice(&[0x6b, 3, 0b0000]);
    }
    shift_dr_command.push(0x87);

    jtag_adapter
        .device
        .write_all(&shift_dr_command)
        .map_err(FtdiError::from)?;

    let mut response: Vec<u8> = vec![];
    jtag_adapter
        .device
        .read_to_end(&mut response)
        .map_err(FtdiError::from)?;

    debug!("resp {:02x?}", response);

    let usercode = response
        .iter()
        .rev()
        .fold(0, |acc, d| acc << 4 | (*d >> 4) as u32);

    println!("usercode {:#010x}", usercode);
    assert_eq!(usercode, 0x12345678);
    Ok(())
}
