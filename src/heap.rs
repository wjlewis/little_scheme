use crate::data::{SchemeObj, Tag};
use crate::memory::{Header, Mem, MemRead, MemWrite};

pub struct Heap {
    space: Vec<u8>,
    get_roots: Box<dyn Fn() -> Vec<usize>>,
}

impl Mem for Heap {
    fn write(&mut self, addr: usize, datum: u8) {
        self.space[addr] = datum;
    }

    fn read(&self, addr: usize) -> u8 {
        self.space[addr]
    }

    fn alloc<T: MemWrite>(&mut self, obj: &T) -> usize {
        self.alloc_bytes(obj.size(), true)
    }
}

// TODO Implement `Iterator` for `Heap`.
impl Heap {
    pub fn new(size: usize, get_roots: Box<dyn Fn() -> Vec<usize>>) -> Heap {
        let space = vec![0; size];
        let mut mem = Heap { space, get_roots };

        // IMPORTANT We initialize this header's `size` to the entire
        // size of the memory we have. However, this isn't correct: we
        // need to subtract the size of the header itself. However, this
        // is easiest to do _after_ the header has already been created.
        let mut header = Header::new(0, size, false);
        let header_size = header.size();
        header.size = size - header_size;
        header.write(&mut mem, 0);

        mem
    }

    /// # Notes
    ///
    /// Returns the address of the first byte _within_ the allocated
    /// block, and *not* the address of the block header.
    fn alloc_bytes(&mut self, n: usize, attempt_collect: bool) -> usize {
        let mut header: Header;
        let mut header_addr = 0;

        loop {
            header = Header::read(self, header_addr);

            if !header.allocd && header.size >= n {
                self.alloc_block(&mut header, n);
                header.write(self, header_addr);
                return header_addr + header.size();
            }

            if header.next == 0 {
                if attempt_collect {
                    self.collect();
                    return self.alloc_bytes(n, false);
                }

                panic!("Unable to allocate: out of memory");
            }

            header_addr = header.next;
        }
    }

    /// Marks the block headed by `header` as allocated, and -- if the
    /// block is large enough -- splits it into two blocks where the
    /// second is unallocated.
    ///
    /// # Notes
    ///
    /// We still need to write our updated (original) header to memory,
    /// via `header.write(..)`, in order to persist the changes we've
    /// made to it. At the moment, we do this in the caller
    /// (`self.alloc_bytes`), but it may make more sense to do it here.
    fn alloc_block(&mut self, header: &mut Header, n: usize) {
        header.allocd = true;

        if header.size >= n + header.size() {
            let residue_size = header.size - n;
            let residue_addr = (header.next + self.space.len() - residue_size) % self.space.len();

            let residue_header = Header::new(header.next, residue_size - header.size(), false);

            header.size = n;
            header.next = residue_addr;

            residue_header.write(self, residue_addr);
        }
    }

    fn collect(&mut self) {
        self.mark();
        self.sweep();
    }

    fn mark(&mut self) {
        let header_size = Header::new(0, 0, false).size();

        let mut root_addrs = (*self.get_roots)();

        while let Some(root_addr) = root_addrs.pop() {
            let header_addr = root_addr - header_size;
            let mut header = Header::read(self, header_addr);

            if header.marked {
                continue;
            }

            header.marked = true;
            header.write(self, header_addr);

            root_addrs.append(&mut children(self, root_addr));
        }
    }

    fn sweep(&mut self) {
        let mut header: Header;
        let mut header_addr = 0;

        loop {
            header = Header::read(self, header_addr);

            if !header.marked {
                // Coalesce with following unmarked blocks
                let mut next = header.next;
                loop {
                    let next_header = Header::read(self, next);
                    if next_header.marked {
                        break;
                    }

                    next = next_header.next;
                }

                header.allocd = false;
                header.next = next;
            }

            header.marked = false;
            header.write(self, header_addr);

            if header.next == 0 {
                break;
            }

            header_addr = header.next;
        }
    }
}

/// Returns the addresses of any child objects that are part of the
/// parent object stored at `parent_addr`.
pub fn children<M: Mem>(mem: &M, parent_addr: usize) -> Vec<usize> {
    match Tag::from(mem.read(parent_addr)) {
        Tag::Pair => {
            let prim_size = SchemeObj::Nil.size();
            let car_addr = parent_addr + 1;
            let cdr_addr = parent_addr + 1 + prim_size;

            let mut children = vec![];

            if Tag::from(mem.read(car_addr)) == Tag::Box {
                children.push(usize::read(mem, car_addr + 1));
            }
            if Tag::from(mem.read(cdr_addr)) == Tag::Box {
                children.push(usize::read(mem, cdr_addr + 1));
            }

            children
        }
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initalize() {
        let mem = Heap::new(32, Box::new(|| vec![]));
        let header = Header::read(&mem, 0);
        assert_eq!(header, Header::new(0, 32 - header.size(), false));
    }

    #[test]
    fn alloc_split() {
        let size = 128;
        let mut mem = Heap::new(size, Box::new(|| vec![]));

        let n = 12;
        let addr = mem.alloc_bytes(n, false);

        let header1 = Header::read(&mem, 0);
        let header2 = Header::read(&mem, header1.size() + n);

        assert_eq!(addr, header1.size());
        assert_eq!(header1, Header::new(header1.size() + n, n, true));
        assert_eq!(
            header2,
            Header::new(0, size - n - 2 * header1.size(), false)
        );
    }

    #[test]
    fn alloc_no_split() {
        let test_header = Header::new(0, 0, false);
        let n = 43;

        let mut mem = Heap::new(test_header.size() + n, Box::new(|| vec![]));

        let addr = mem.alloc_bytes(n, false);

        let header1 = Header::read(&mem, 0);

        assert_eq!(addr, header1.size());
        assert_eq!(header1.next, 0);
    }

    #[test]
    #[should_panic]
    fn alloc_too_big() {
        let mut mem = Heap::new(10, Box::new(|| vec![]));

        mem.alloc_bytes(123, false);
    }
}
