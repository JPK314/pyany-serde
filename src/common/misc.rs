pub fn get_bytes_to_alignment<T>(addr: usize) -> usize {
    let alignment = align_of::<T>();
    let aligned_addr = addr.wrapping_add(alignment - 1) & 0usize.wrapping_sub(alignment);
    aligned_addr.wrapping_sub(addr)
}
