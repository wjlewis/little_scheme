use super::{Mem, MemRead, MemWrite};
use std::mem::size_of;

/// "Auxiliary" trait implementations and other goodies. In particular,
/// this module includes implementations of `MemRead` and `MemWrite` for
/// `usize`, `isize`, and other primitives.

impl MemRead for usize {
    /// Read a `usize` as a *little-endian* encoded sequence of bytes.
    ///
    /// # Notes
    ///
    /// The number of bytes occupied by a `usize` is
    /// architecture-dependent, so this implementation must conspire
    /// with our implementation of `MemWrite` to use the same number of
    /// bytes. This is easy: we just check
    /// `std::mem::size_of::<usize>()` in both implementations, as use
    /// that to determine the number of bytes to use.
    ///
    /// Also, because I always forget the ordering associated with
    /// endianness, here is an explicit example:
    ///
    /// ```ignore
    ///    +------+------+------+------+------+------+------+------+
    /// .. | 0x00 | 0x11 | 0x22 | 0x33 | 0x44 | 0x55 | 0x66 | 0x77 | ..
    ///    +------+------+------+------+------+------+------+------+
    ///       ^LSB                                             ^MSB
    /// ```
    ///
    /// In this case (assuming `std::mem::size_of::<usize>()` is 8), the
    /// `usize` we'd read is equal to:
    ///
    /// ```ignore
    /// 0x77_66_55_44_33_22_11_00
    /// ```
    fn read<M: Mem>(mem: &M, addr: usize) -> usize {
        let mut out: usize = 0;

        for i in 0..size_of::<usize>() {
            out |= (mem.read(addr + i) as usize) << (i * 8);
        }

        out
    }
}

impl MemWrite for usize {
    /// Writes a `usize` as a *little-endian* encoded sequence of bytes.
    ///
    /// See the documentation for the implementation of `MemRead` for
    /// more information.
    fn write<M: Mem>(&self, mem: &mut M, addr: usize) {
        for i in 0..size_of::<usize>() {
            let byte = (self >> (i * 8) & 0xFF) as u8;

            mem.write(addr + i, byte);
        }
    }

    fn size(&self) -> usize {
        size_of::<usize>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_usize() {
        let mem: Vec<u8> = vec![
            0x00, 0x00, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x00, 0x00,
        ];

        let expected = if size_of::<usize>() == 8 {
            0x77_66_55_44_33_22_11_00
        } else {
            0x33_22_11_00
        };

        assert_eq!(usize::read(&mem, 2), expected);
    }

    #[test]
    fn write_usize() {
        let mut mem: Vec<u8> = vec![0x00; 14];

        let bytes: usize = if size_of::<usize>() == 8 {
            0x77_66_55_44_33_22_11_00
        } else {
            0x33_22_11_00
        };

        bytes.write(&mut mem, 5);

        if size_of::<usize>() == 8 {
            assert_eq!(
                &mem[5..13],
                [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77]
            );
        } else {
            assert_eq!(&mem[5..9], [0x00, 0x11, 0x22, 0x33]);
        };
    }

    #[test]
    fn write_read_usize() {
        let mut mem: Vec<u8> = vec![0x00; 23];

        let bytes: usize = if size_of::<usize>() == 8 {
            0x34_a5_88_9f_31_90_93_ea
        } else {
            0x31_90_93_ea
        };

        let addr = 4;

        bytes.write(&mut mem, addr);

        assert_eq!(usize::read(&mem, addr), bytes);
    }
}
