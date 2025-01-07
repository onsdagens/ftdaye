// jtag helpers for ftdi mpsse

use crate::ftdaye::mpsse::{
    cmd_read_imm, cmd_read_write_imm, cmd_write_imm, Clock_Data_Bits_Out_on_neg_ve_LSB_first,
    Clock_Data_to_TMS_on_neg_ve_LSB_first, CmdImm,
};
use crate::ftdaye::{BitMode, Device};

use log::*;
use std::io::{Read, Write};

#[derive(Debug)]
#[allow(dead_code)]
pub struct FtdiMpsse {
    pub device: Device,
    buffer_size_bytes: u16,
    actual_speed_khz: u16,
}

// Todo: stateful FtdiMpsse?
#[allow(dead_code)]
enum JtagState {
    Rti,
    ShiftDr(u8),
}

// Todo: what kind of errors do we want here?
// for now, just unwrap and panic...
impl FtdiMpsse {
    pub fn new(mut device: Device, speed_khz: u32) -> Self {
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

        // FTDI 2232
        // Disable divide-by-5 mode
        device.disable_divide_by_5().unwrap();
        let buffer_size_bytes: u16 = 4096;
        let max_clock_khz: u32 = 30_000;

        // If `speed_khz` is not a divisor of the maximum supported speed, we need to round up
        let is_exact = max_clock_khz % speed_khz == 0;

        // If `speed_khz` is 0, use the maximum supported speed
        let divisor =
            (max_clock_khz.checked_div(speed_khz).unwrap_or(1) - is_exact as u32).min(0xFFFF);

        let actual_speed_khz = (max_clock_khz / (divisor + 1)) as u16;

        info!(
            "Setting speed to {} kHz (divisor: {}, actual speed: {} kHz)",
            speed_khz, divisor, actual_speed_khz
        );

        device.configure_clock_divider(divisor as u16).unwrap();

        device.disable_loopback().unwrap();

        // check bad command
        let bad_command = [0xAB];
        device.write_all(&bad_command).unwrap();

        let mut junk = vec![];
        let r = device.read_to_end(&mut junk);

        debug!("r {:?}, buf {:x?}", r, junk);
        Self {
            device,
            buffer_size_bytes,
            actual_speed_khz,
        }
    }

    pub fn read_write_register(&mut self, ir: u8, data: &mut [u8]) {
        debug!("read write ir #{:#04x}", ir);
        self.rti_to_shift_ir();
        self.shift_ir(ir);

        self.rti_to_shift_dr();

        self.device.write(&cmd_read_write_imm(data)).unwrap();

        self.device.read_exact(data).unwrap();

        self.dr_to_rti();
    }

    pub fn write_register(&mut self, ir: u8, data: &[u8]) {
        debug!("write ir #{:#04x}", ir);
        self.rti_to_shift_ir();
        self.shift_ir(ir);

        self.rti_to_shift_dr();

        self.device.write(&cmd_write_imm(data)).unwrap();

        self.dr_to_rti();
    }

    pub fn assert_ftdi_buffer_empty(&mut self) {
        let mut junk = vec![];
        let _ = self.device.read_to_end(&mut junk);
        assert!(junk.len() == 0, "buffer not empty {:?}", junk)
    }

    pub fn read_register(&mut self, ir: u8, data: &mut [u8]) {
        debug!("read ir #{:#04x}", ir);
        self.rti_to_shift_ir();
        self.shift_ir(ir);

        self.rti_to_shift_dr();

        self.device.write(&cmd_read_imm(data.len())).unwrap();

        self.device.read_exact(data).unwrap();

        self.dr_to_rti();
    }

    // reset state machine, and go to rti
    pub fn reset_and_to_rti(&mut self) {
        self.device
            .write(&[Clock_Data_to_TMS_on_neg_ve_LSB_first, 4, 0b1_1111, CmdImm])
            .unwrap();
        self.device
            .write(&[Clock_Data_to_TMS_on_neg_ve_LSB_first, 0, 0b0, CmdImm])
            .unwrap();
    }

    // go from rti to shift dr
    pub fn rti_to_shift_dr(&mut self) {
        self.device
            .write(&[Clock_Data_to_TMS_on_neg_ve_LSB_first, 2, 0b001, CmdImm])
            .unwrap();
    }

    // go from rti to shift ir
    pub fn rti_to_shift_ir(&mut self) {
        self.device
            .write(&[Clock_Data_to_TMS_on_neg_ve_LSB_first, 3, 0b0011, CmdImm])
            .unwrap();
    }

    // go from dr back to rti
    pub fn dr_to_rti(&mut self) {
        self.device
            .write(&[Clock_Data_to_TMS_on_neg_ve_LSB_first, 2, 0b011, CmdImm])
            .unwrap();
    }

    // go from ir back to rti
    pub fn ir_to_rti(&mut self, bit7: u8) {
        self.device
            .write(&[
                Clock_Data_to_TMS_on_neg_ve_LSB_first,
                2,
                bit7 | 0b011,
                CmdImm,
            ])
            .unwrap();
    }

    // shift ir and go back to rti
    pub fn shift_ir(&mut self, ir: u8) {
        // 5 bits of ir
        self.device
            .write(&[Clock_Data_Bits_Out_on_neg_ve_LSB_first, 4, ir, CmdImm])
            .unwrap();
        // msb of ir as bit 7 of next transaction
        self.ir_to_rti((ir & 0b10_0000) << 2);
    }
}
