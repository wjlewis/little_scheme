mod aux;
mod header;

use header::Header;

pub struct Memory {
    space: Vec<u8>,
}

impl Mem for Memory {
    fn write(&mut self, addr: usize, datum: u8) {
        todo!()
    }

    fn read(&self, addr: usize) -> u8 {
        todo!()
    }

    fn alloc<T: MemWrite>(&mut self, obj: &T) -> usize {
        todo!()
    }
}

impl Memory {
    pub fn new(size: usize) -> Memory {
        let space = vec![0; size];
        let mut mem = Memory { space };

        // IMPORTANT We initialize this header's `size` to the entire
        // size of the memory we have. However, this isn't correct: we
        // need to subtract the size of the header itself. However, this
        // is easiest to do _after_ the header has already been created.
        let mut header = Header::new(0, size, false);
        let header_size = header.size();
        header.set_size(size - header_size);
        header.write(&mut mem, 0);

        mem
    }
}

/// Represents a memory store as a "sink of bytes". This entails two
/// capabilities: writing a byte to a specific location, and reading the
/// byte at a specific location.
pub trait Mem {
    /// Allocate space for `obj`, and return a pointer to the
    /// freshly-allocated bytes.
    fn alloc<T: MemWrite>(&mut self, obj: &T) -> usize;

    /// Write the provided byte to the location indicated by `addr`.
    fn write(&mut self, addr: usize, datum: u8);

    /// Read the byte at the location indicated by `addr`.
    ///
    /// # Panics
    ///
    /// `read` may (and does, in our instances) panic if `addr` is not a
    /// valid memory location. Such a situation is analagous to a
    /// segmentation fault, and represents a logical error in an
    /// implementation of the `MemRead` trait.
    fn read(&self, addr: usize) -> u8;
}

/// Represents the capability for an object to by read from a "sink of
/// bytes". This is essentially a specialized deserialization trait.
pub trait MemRead {
    fn read<M: Mem>(mem: &M, addr: usize) -> Self;
}

/// Represents the capability for an object to be written to a "sink of
/// bytes". It is the dual of `MemRead`, and is essentially a
/// specialized serialization trait.
pub trait MemWrite {
    fn write<M: Mem>(&self, mem: &mut M, addr: usize);

    /// Returns the number of bytes required to represent `self` in
    /// memory.
    ///
    /// # Notes
    ///
    /// This is used to determine how much space to allocate for a
    /// particular object.
    fn size(&self) -> usize;
}
