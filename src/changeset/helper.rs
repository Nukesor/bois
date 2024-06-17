/// Check whether two unix mode permissions are identical.
///
/// The modes that're returned from Rust's std `mode` function contains the file type bits
/// that're contained in the 16bit mode value.
///
/// The structure is:
/// 4-bit object type
/// 3-bit (1 octal) special bits.
/// 9-bit (three octals) unix permission.
///
/// 0000 - 000 - 000_000_000
pub fn equal_permissions(mut one: u32, mut two: u32) -> bool {
    // Remove the filetype mode bits, as we're not interested in them
    let mask: u32 = 0b0000_0000_0000_0000_0000_1111_1111_1111;
    one &= mask;
    two &= mask;

    one == two
}
