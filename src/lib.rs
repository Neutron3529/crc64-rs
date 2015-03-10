#![feature(core)]

/// Calculate the crc64 checksum of the given data, starting with the given crc.
///
/// Implements the CRC64 used by Redis, which is the variant with "Jones" coefficients and init value of 0.
///
/// Specification of this CRC64 variant follows:
///
/// ```text
/// Name: crc-64-jones
/// Width: 64 bites
/// Poly: 0xad93d23594c935a9
/// Reflected In: True
/// Xor_In: 0xffffffffffffffff
/// Reflected_Out: True
/// Xor_Out: 0x0
/// Check("123456789"): 0xe9c6d914c4b8d9ca
/// ```
///
/// Example:
///
/// ```rust
/// crc64::crc64(0, "123456789".as_bytes());
/// ```

use std::mem;
use crc_table::CRC64_TAB;
mod crc_table;

fn crc_reflect(data: u64, len: usize) -> u64 {
    let mut data = data;
    let mut ret = data & 0x01;

    let mut i = 1usize;
    while i < len {
        data >>= 1;
        ret = (ret << 1) | (data & 0x01);
        i+=1;
    }

    ret
}

fn crc64_trivial(crc: u64, in_data: &[u8]) -> u64 {
    let mut crc = crc;
    let len = in_data.len();

    let poly : u64 = 0xad93d23594c935a9;

    let mut bit : bool;

    let mut offset = 0usize;

    while offset < len {
        let c = in_data[offset];
        let mut i = 0x01;
        while i & 0xff != 0x0 {
            bit = crc & 0x8000000000000000 != 0x0;
            if c & i != 0x0 {
                bit = !bit;
            }
            crc <<= 1;
            if bit {
                crc ^= poly;
            }
            i <<= 1;
        }
        crc &= 0xffffffffffffffff;
        offset+=1;
    }
    crc = crc & 0xffffffffffffffff;

    return crc_reflect(crc, 64) ^ 0x0000000000000000;
}

pub fn crc64_init() -> Vec<Vec<u64>> {
    let mut crc : u64;

    let mut table : Vec<Vec<u64>> = Vec::with_capacity(8);

    for _ in (0..8) {
        table.push(Vec::with_capacity(256));
    };

    for n in (0usize..256) {
        table[0].push(crc64_trivial(0, vec![n as u8].as_slice()));
        table[1].push(0);
        table[2].push(0);
        table[3].push(0);
        table[4].push(0);
        table[5].push(0);
        table[6].push(0);
        table[7].push(0);
    }

    for n in (0usize..256) {
        crc = table[0][n];
        for k in (1usize..8) {
            let idx  = (crc as usize) & 0xff;
            crc = table[0][idx] ^ (crc >> 8);
            table[k][n] = crc;
        }
    };

    table
}

// transmute slice of 8 u8 values to one u64 (drop the length)
macro_rules! slice_to_long {
    ($curVec:expr) => {
        {
            unsafe {
                let (tmp, _) : (*const u64, u64) = mem::transmute(&$curVec);
                *tmp
            }
        }
    }
}

pub fn crc64(crc: u64, data: &[u8]) -> u64 {
    let mut crc = crc;
    let mut len = data.len();
    let mut offset = 0usize;

    while len >= 8 {
        crc ^= slice_to_long!(data[offset..(offset+8)]);
        crc = CRC64_TAB[7][(crc & 0xff) as usize] ^
              CRC64_TAB[6][((crc >> 8) & 0xff) as usize] ^
              CRC64_TAB[5][((crc >> 16) & 0xff) as usize] ^
              CRC64_TAB[4][((crc >> 24) & 0xff) as usize] ^
              CRC64_TAB[3][((crc >> 32) & 0xff) as usize] ^
              CRC64_TAB[2][((crc >> 40) & 0xff) as usize] ^
              CRC64_TAB[1][((crc >> 48) & 0xff) as usize] ^
              CRC64_TAB[0][(crc >> 56) as usize];

        offset += 8;
        len -= 8;
    }

    while len > 0 {
        crc = CRC64_TAB[0][((crc ^ data[offset] as u64) & 0xff) as usize] ^ (crc >> 8);
        offset += 1;
        len -= 1;
    }

    crc
}

#[test]
fn test_crc64_works() {
    assert_eq!(0xe9c6d914c4b8d9ca, crc64(0, "123456789".as_bytes()))
}
