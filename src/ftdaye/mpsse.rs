#![allow(non_upper_case_globals)]

pub const fn cmd_shift(
    neg_ve_clk_read: bool,
    bit_mode: bool,
    neg_ve_clk_write: bool,
    lsb_first: bool,
    write_tdi: bool,
    read_tdo: bool,
    write_tms: bool,
) -> u8 {
    (neg_ve_clk_read as u8) << 0
        | (bit_mode as u8) << 1
        | (neg_ve_clk_write as u8) << 2
        | (lsb_first as u8) << 3
        | (write_tdi as u8) << 4
        | (read_tdo as u8) << 5
        | (write_tms as u8) << 6
}

// 3.4.9 Clock Data Bytes In and Out LSB first
#[rustfmt::skip]
pub const Clock_Data_Bytes_In_and_Out_LSB_first: u8 = cmd_shift(
    true, 
    false, 
    false, 
    true, 
    true, 
    true, 
    false);

pub const CmdImm: u8 = 0x87;
pub const CmdBadCommand: u8 = 0xAB;

pub fn cmd_write_imm(data: &[u8]) -> Vec<u8> {
    assert!(
        data.len() > 0 && data.len() <= 65536,
        "data length is {} must be in range 1..=65536 ",
        data.len()
    );
    let len = data.len() - 1;
    let mut v = vec![
        Clock_Data_Bytes_In_and_Out_LSB_first,
        len as u8,
        (len >> 8) as u8,
    ];
    v.extend_from_slice(data);
    v.push(CmdImm);
    // println!("{:x?}", v);
    v
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cmd_write_imm_min() {
        assert_eq!(&cmd_write_imm(&[0x12]), &[0x39, 0, 0, 0x12, 0x87]);
    }

    #[test]
    fn test_cmd_write_imm_max() {
        let cmd = cmd_write_imm(&[0u8; 65536]);
        assert_eq!(cmd[1], 0xff);
        assert_eq!(cmd[2], 0xff);
    }
    #[test]
    #[should_panic]
    fn test_cmd_write_imm_0() {
        let _ = cmd_write_imm(&[]);
    }
    #[test]
    #[should_panic]
    fn test_cmd_write_imm_too_large() {
        let _ = cmd_write_imm(&[0u8; 65537]);
    }
}
