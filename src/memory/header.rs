use super::{Mem, MemRead, MemWrite};
use std::mem::size_of;

/// Represents a header for a block of memory. Each header includes a
/// pointer to the next block (`next`), its size (`size`), and several
/// flags indicating if the block has been allocated (`allocd`), or if
/// the block has been marked as in use during a marking phase
/// (`marked`).
#[derive(Debug, PartialEq)]
struct Header {
    next: usize,
    size: usize,
    allocd: bool,
    marked: bool,
}

impl MemRead for Header {
    fn read<M: Mem>(mem: &M, addr: usize) -> Header {
        let word_size = size_of::<usize>();

        let next = usize::read(mem, addr);
        let size = usize::read(mem, addr + word_size);
        let flags = mem.read(addr + 2 * word_size);

        let allocd = flags & 0b1000_0000 > 0;
        let marked = flags & 0b0100_0000 > 0;

        Header {
            next,
            size,
            allocd,
            marked,
        }
    }
}

impl MemWrite for Header {
    fn write<M: Mem>(&self, mem: &mut M, addr: usize) {
        let word_size = size_of::<usize>();

        self.next.write(mem, addr);
        self.size.write(mem, addr + word_size);

        let allocd_flag = if self.allocd { 0b1000_0000 } else { 0 };
        let marked_flag = if self.marked { 0b0100_0000 } else { 0 };

        let flags = allocd_flag | marked_flag;

        mem.write(addr + 2 * word_size, flags);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_read_header() {
        let mut mem: Vec<u8> = vec![0x00; 128];

        let header = Header {
            next: 2451423,
            size: 7813423,
            allocd: true,
            marked: false,
        };
        let addr = 34;

        header.write(&mut mem, addr);

        assert_eq!(Header::read(&mem, addr), header);
    }
}