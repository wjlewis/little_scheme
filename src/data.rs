use super::memory::{Mem, MemRead, MemWrite};
use std::mem::size_of;

/// Represents an object that can be written to and read from our
/// memory. Such objects have no "semantics" associated with them. That
/// is to say, a `Pair` may represent a value, or part of a closure, or
/// part of an environment.
pub enum SchemeObj {
    Nil,
    Bool(bool),
    Number(isize),
    Symbol(usize),
    Pair {
        car: Box<SchemeObj>,
        cdr: Box<SchemeObj>,
    },
}

impl MemRead for SchemeObj {
    fn read<M: Mem>(mem: &M, addr: usize) -> SchemeObj {
        let tag = Tag::from(mem.read(addr));

        use SchemeObj::*;

        match tag {
            Tag::Nil => Nil,
            Tag::Bool => {
                let value = if mem.read(addr + 1) == 0 { false } else { true };
                Bool(value)
            }
            Tag::Number => Number(usize::read(mem, addr + 1) as isize),
            Tag::Symbol => Symbol(usize::read(mem, addr + 1)),
            Tag::Pair => {
                let car = Box::read(mem, addr + 1);
                let cdr = Box::read(mem, addr + 1 + car.size());

                Pair { car, cdr }
            }
            Tag::Box => panic!("Attempted to read Box at {}", addr),
        }
    }
}

impl MemWrite for SchemeObj {
    fn write<M: Mem>(&self, mem: &mut M, addr: usize) {
        use SchemeObj::*;

        match self {
            Nil => mem.write(addr, u8::from(Tag::Nil)),
            Bool(b) => {
                mem.write(addr, u8::from(Tag::Bool));

                let value = usize::from(*b);
                value.write(mem, addr + 1);
            }
            Number(n) => {
                mem.write(addr, u8::from(Tag::Number));

                let value = usize::from(*n as usize);
                value.write(mem, addr + 1);
            }
            Symbol(i) => {
                mem.write(addr, u8::from(Tag::Symbol));

                (*i).write(mem, addr + 1);
            }
            Pair { car, cdr } => {
                mem.write(addr, u8::from(Tag::Pair));

                car.write(mem, addr + 1);
                cdr.write(mem, addr + 1 + size_of::<usize>());
            }
        }
    }

    fn size(&self) -> usize {
        use SchemeObj::*;

        let prim_size = 1 + size_of::<usize>();

        match self {
            Nil | Bool(_) | Number(_) | Symbol(_) => prim_size,
            Pair { .. } => 1 + 2 * prim_size,
        }
    }
}

impl MemRead for Box<SchemeObj> {
    fn read<M: Mem>(mem: &M, addr: usize) -> Box<SchemeObj> {
        Box::new(SchemeObj::read(mem, addr + 1))
    }
}

impl MemWrite for Box<SchemeObj> {
    fn write<M: Mem>(&self, mem: &mut M, addr: usize) {
        mem.write(addr, u8::from(Tag::Box));
        let obj_addr = mem.alloc(&**self);
        obj_addr.write(mem, addr + 1);
    }

    fn size(&self) -> usize {
        1 + size_of::<usize>()
    }
}

/// Used to indicate the type of object represented by the following
/// bytes in memory.
pub enum Tag {
    Box,
    Nil,
    Bool,
    Number,
    Symbol,
    Pair,
}

impl From<u8> for Tag {
    fn from(byte: u8) -> Tag {
        use Tag::*;

        match byte {
            0 => Box,
            1 => Nil,
            2 => Bool,
            3 => Number,
            4 => Symbol,
            5 => Pair,
            _ => panic!("No tag associated with byte: {}", byte),
        }
    }
}

impl From<Tag> for u8 {
    fn from(tag: Tag) -> u8 {
        use Tag::*;

        match tag {
            Box => 0,
            Nil => 1,
            Bool => 2,
            Number => 3,
            Symbol => 4,
            Pair => 5,
        }
    }
}
