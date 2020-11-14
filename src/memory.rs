mod aux;
mod header;

/// Represents a memory store as a "sink of bytes". This entails two
/// capabilities: writing a byte to a specific location, and reading the
/// byte at a specific location.
pub trait Mem {
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
}

#[cfg(test)]
impl Mem for Vec<u8> {
    fn write(&mut self, addr: usize, byte: u8) {
        self[addr] = byte;
    }

    fn read(&self, addr: usize) -> u8 {
        self[addr]
    }
}
