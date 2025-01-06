// jtag helpers for ftdi mpsse

use crate::ftdaye::mpsse::{cmd_write_imm, CmdImm};
use crate::ftdaye::Device;
use log::*;
use std::io::{Read, Write};

pub struct FtdiMpsse {
    device: Device,
}

enum JtagState {
    Rti,
    ShiftDr(u8),
}

// Todo: what kind of errors do we want here?
// for now, just unwrap and panic...
impl FtdiMpsse {
    pub fn new(device: Device) -> Self {
        Self { device }
    }

    pub fn read_write_register(&mut self, ir: u8, data: &mut [u8]) {
        debug!("read ir #{:#04x}", ir);
        self.rti_to_shift_ir();
        self.shift_ir(ir);

        self.rti_to_shift_dr();

        self.device.write(&cmd_write_imm(data)).unwrap();

        self.device.read_exact(data).unwrap();

        self.dr_to_rti();
    }

    // reset state machine, and go to rti
    pub fn reset_to_rti(&mut self) {
        self.device.write(&[0x4b, 5, 0b11111, CmdImm]).unwrap();
        self.device.write(&[0x4b, 0, 0b0, CmdImm]).unwrap();
    }

    // go from rti to shift dr
    pub fn rti_to_shift_dr(&mut self) {
        self.device.write(&[0x4b, 2, 0b001, CmdImm]).unwrap();
    }

    // go from rti to shift ir
    pub fn rti_to_shift_ir(&mut self) {
        self.device.write(&[0x4b, 3, 0b0011, CmdImm]).unwrap();
    }

    // go from dr back to rti
    pub fn dr_to_rti(&mut self) {
        self.device.write(&[0x4b, 2, 0b011, CmdImm]).unwrap();
    }

    // go from ir back to rti
    pub fn ir_to_rti(&mut self, bit7: u8) {
        self.device.write(&[0x4b, 2, bit7 | 0b011, CmdImm]).unwrap();
    }

    // shift ir and go back to rti
    pub fn shift_ir(&mut self, ir: u8) {
        // 5 bits of ir
        self.device.write(&[0x1b, 4, ir, CmdImm]).unwrap();
        // msb of ir as bit 7 of next transaction
        self.ir_to_rti((ir & 0b10_0000) << 2);
    }
}
