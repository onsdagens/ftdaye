use anyhow::Result;
use bitvec::field::BitField;
use log::*;

use ftdaye::{
    //     // probe::list::Lister,
    command_compacter::Command,
    //     // architecture::arm::{sequences::DefaultArmSequence, ApAddress, DpAddress},
    // ftdaye::Builder,
    // FtdiDevice,
    JtagAdapter,
    FTDI_COMPAT_DEVICES,
};
fn main() -> Result<()> {
    pretty_env_logger::init();

    let device_info = nusb::list_devices()
        .unwrap()
        .find(|dev| dev.vendor_id() == 0x0403 && dev.product_id() == 0x6010)
        .expect("device not connected");

    let mut jtag_adapter = JtagAdapter::open(FTDI_COMPAT_DEVICES[0], device_info)?;
    debug!("open {:?}", jtag_adapter);

    // jtag_adapter.apply_clock_speed(1000)?;

    jtag_adapter.attach()?;
    debug!("attach {:?}", jtag_adapter);

    // jtag_adapter.set_speed_khz(1000);

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

    // flush command
    jtag_adapter.flush()?;
    debug!("TLR flush {:x?}", jtag_adapter);

    // // ir shift state
    // let to_shift_ir_command = Command::TmsBits {
    //     bit_count: 5,
    //     // tms_bits: 0b0100, // DR shift
    //     tms_bits: 0b00110, // to IR shift
    //     // tms_bits: 0b01100, // IR shift
    //     tdi: false,
    //     capture: false,
    // };

    // jtag_adapter.append_command(to_shift_ir_command)?;
    // debug!("To Shift IR {:x?}", jtag_adapter);

    // // flush command
    // jtag_adapter.flush()?;
    // debug!("To Shift IR flush {:x?}", jtag_adapter);

    // // write IDCODE register
    // let tdi_09_command = Command::TdiBits {
    //     bit_count: 6,
    //     tdi_bits: 0b1001_00,
    //     // tdi_bits: 0b00_1001
    //     capture: false,
    // };

    // jtag_adapter.append_command(tdi_09_command)?;
    // debug!("Tdi command {:x?}", jtag_adapter);

    // // flush command
    // jtag_adapter.flush()?;
    // debug!("Tdi flush {:x?}", jtag_adapter);

    // // select dr
    // let select_dr_command = Command::TmsBits {
    //     bit_count: 3,
    //     tms_bits: 0b111, // IR shift
    //     tdi: false,
    //     capture: false,
    // };
    // jtag_adapter.append_command(select_dr_command)?;
    // debug!("select dr {:x?}", jtag_adapter);

    // // flush command
    // jtag_adapter.flush()?;
    // debug!("select dr flush {:x?}", jtag_adapter);

    // put in dr shift state from tlr
    let to_shift_dr_command = Command::TmsBits {
        bit_count: 4,
        tms_bits: 0b0010, // DR shift
        tdi: false,
        capture: false,
    };
    jtag_adapter.append_command(to_shift_dr_command)?;
    debug!("To Shift DR {:x?}", jtag_adapter);

    // flush command
    jtag_adapter.flush()?;
    debug!("To Shift DR flush {:x?}", jtag_adapter);

    debug!("-------");
    for i in 0..8 {
        // shift dr, assume < 8 bits to read
        let shift_dr_command = Command::TmsBits {
            bit_count: 4,
            tms_bits: 0b000000, // DR shift
            tdi: false,
            capture: true,
        };
        jtag_adapter.append_command(shift_dr_command)?;
        debug!("Shift DR {:x?}", jtag_adapter);

        // flush command
        jtag_adapter.flush()?;
        debug!("Shift DR flush {:x?}", jtag_adapter);
    }

    let id_code_bits = jtag_adapter.read_captured_bits()?;
    debug!("cp {:?}", id_code_bits);

    let idcode: u32 = id_code_bits.load();

    debug!("idcode {:#10x}", idcode);
    assert_eq!(idcode, 0x0362_d093);
    Ok(())
}
