/// Align `addr` downwards to the nearest multiple of `align`.
///
/// The returned usize is always <= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2.
pub fn align_down(addr: usize, align: usize) -> usize {
    let count = align.count_ones();
    if count != 1 {
        panic!("Alignment not a power of 2");
    }
    if addr < align {
        0
    } else if (addr - align) % align != 0 {
        addr - (addr % align)
    } else {
        addr
    }
}

/// Align `addr` upwards to the nearest multiple of `align`.
///
/// The returned `usize` is always >= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2
/// or aligning up overflows the address.
pub fn align_up(addr: usize, align: usize) -> usize {
    let count = align.count_ones();
    if count != 1 {
        panic!("Alignment not a power of 2");
    }
    if addr + align < addr {
        panic!("Overflow");
    } else if (addr + align) % align != 0 {
        addr + (align - (addr % align))
    } else {
        addr
    }

}
