use core::slice;

use super::err::CatscopeWasmError;

pub const DATA_LIST: [usize; 20] = [
    8,
    16,
    64,
    128,
    256,
    512,
    728,
    1024,
    2048,
    5 * 1024,
    30 * 1024,
    100 * 1024,
    500 * 1024,
    1024 * 1024,
    5 * 1024 * 1024,
    10 * 1024 * 1024,
    20 * 1024 * 1024,
    50 * 1024 * 1024,
    100 * 1024 * 1024,
    500 * 1024 * 1024,
];

/// Find the smallest byte slice size that encompasses the target slice.
pub fn find_k(arr: &[usize], target: usize) -> Option<usize> {
    let n = arr.len();
    if arr[n - 1] < target {
        return None;
    }
    if target < arr[0] {
        return Some(0);
    }
    let (mut low, mut high) = (0, n - 1);
    while low <= high {
        let mut mid = high - low;
        if mid % 2 == 0 {
            mid = low + mid / 2
        } else {
            mid = low + (mid + 1) / 2
        }
        // warn!(
        //    "1 - {} {} {}",
        //    DATA_LIST[low], DATA_LIST[mid], DATA_LIST[high]
        // );
        if arr[mid] == target {
            // k falls between arr[mid] and itself
            //        warn!("2 - ");
            return Some(mid + 1);
        } else if arr[mid] < target {
            //       warn!("3 - ");
            low = mid + 1;
        } else {
            //      warn!("4 - ");
            high = mid - 1;
        }
    }
    if arr[high] < target {
        if arr.len() <= high + 1 {
            None
        } else {
            Some(high + 1)
        }
    } else {
        //warn!("5");
        Some(high)
    }
}

pub fn slice_to_struct<T>(slice: &[u8], offset: usize) -> Result<&T, CatscopeWasmError> {
    let t_len = std::mem::size_of::<T>();
    if slice.len() < t_len + offset {
        return Err(CatscopeWasmError::InsufficientBuffer);
    }
    let subbuf = &slice[offset..(offset + t_len)];
    let my_struct: &T = unsafe { &*(subbuf.as_ptr() as *const T) };
    Ok(my_struct)
}

pub trait Slottable {
    fn track_slot(&self) -> u64;
}
pub enum InsertPosition {
    Before,
    After,
    Inside(usize),
    On(usize),
}
pub fn binary_search<T: Slottable>(arr: &[T], target: u64) -> InsertPosition {
    let mut left = 0;
    let mut right = arr.len();
    if arr.is_empty() {
        return InsertPosition::After;
    }
    if target < arr.first().unwrap().track_slot() {
        return InsertPosition::Before;
    }
    if arr.last().unwrap().track_slot() < target {
        return InsertPosition::After;
    }
    while left < right {
        let mid = left + (right - left) / 2;

        if arr[mid].track_slot() == target {
            return InsertPosition::On(mid);
        } else if arr[mid].track_slot() < target {
            left = mid + 1;
        } else {
            right = mid;
        }
    }
    InsertPosition::Inside(left)
}

/// Cast a byte array to a struct.
/// # Safety
pub unsafe fn ptr_to_struct_v2<'a, T>(ptr: *mut T) -> Option<&'a mut T> {
    if ptr.is_null() {
        return None;
    }
    let s = unsafe { ptr.as_mut().unwrap() };
    Some(s)
}

/// Cast a byte array to a read only struct.
/// # Safety
pub unsafe fn ptr_to_struct_read_only_v2<'a, T>(ptr: *const T) -> Option<&'a T> {
    if ptr.is_null() {
        return None;
    }
    let s = unsafe { ptr.as_ref().unwrap() };
    Some(s)
}
pub fn ptr_to_struct<'a, T>(buffer_ptr: u64) -> Option<&'a mut T> {
    let ptr = buffer_ptr as *mut T;
    if ptr.is_null() {
        return None;
    }
    let s = unsafe { ptr.as_mut().unwrap() };
    Some(s)
}
pub fn ptr_to_struct_read_only<'a, T>(buffer_ptr: u64) -> Option<&'a T> {
    let ptr = buffer_ptr as *const T;
    if ptr.is_null() {
        return None;
    }
    let s = unsafe { ptr.as_ref().unwrap() };
    Some(s)
}

pub const PUBKEY_LEN: usize = 32;
pub const U32_LEN: usize = 4;
pub const U64_LEN: usize = 8;

pub fn cast_bytes_manual<T>(bytes: &[u8]) -> &[T] {
    assert_eq!(bytes.len() % size_of::<T>(), 0);
    assert_eq!(bytes.as_ptr() as usize % align_of::<T>(), 0); // unsafe if misaligned

    unsafe { slice::from_raw_parts(bytes.as_ptr() as *const T, bytes.len() / size_of::<T>()) }
}

pub fn match_discriminator(a: &[u8], b: &[u8]) -> bool {
    if a.len() < 8 {
        return false;
    }
    if b.len() < 8 {
        return false;
    }
    for i in 0..8 {
        if a[i] != b[i] {
            return false;
        }
    }
    true
}
