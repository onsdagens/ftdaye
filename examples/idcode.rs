use anyhow::Result;
use bitvec::field::BitField;
use log::*;

use ftdaye::{command_compacter::Command, JtagAdapter, FTDI_COMPAT_DEVICES};
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

    // TLR
    let tlr_command = Command::TmsBits {
        bit_count: 5,
        tms_bits: 0b11111,
        tdi: false,
        capture: false,
    };

    // ensure that we are in TLR state
    jtag_adapter.append_command(tlr_command)?;
    debug!("TLR {:x?}", jtag_adapter);

    // put in dr shift state from tlr
    let to_shift_dr_command = Command::TmsBits {
        bit_count: 4,
        tms_bits: 0b0010, // DR shift
        tdi: false,
        capture: false,
    };
    jtag_adapter.append_command(to_shift_dr_command)?;
    debug!("To Shift DR {:x?}", jtag_adapter);

    // shift dr, assume < 8 bits to read
    let shift_dr_command = Command::TmsBits {
        bit_count: 4,
        tms_bits: 0b0000, // DR shift
        tdi: false,
        capture: true,
    };

    for _ in 0..8 {
        jtag_adapter.append_command(shift_dr_command.clone())?;
        debug!("Shift DR {:x?}", jtag_adapter);
    }

    // flush command
    jtag_adapter.flush()?;
    debug!("Shift DR flush {:x?}", jtag_adapter);

    let id_code_bits = jtag_adapter.read_captured_bits()?;
    debug!("cp {:?}", id_code_bits);

    // convert BitVec via BitField to integral (u32)
    let idcode: u32 = id_code_bits.load();

    debug!("idcode {:#010x}", idcode);
    assert_eq!(idcode, 0x0362_d093);
    Ok(())
}
