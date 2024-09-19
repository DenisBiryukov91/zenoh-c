use std::{
    os::raw::c_void,
    time::{Instant, SystemTime},
};

use chrono::{DateTime, Local};
use libc::c_char;

pub use crate::z_clock_t;
use crate::{
    transmute::{LoanedCTypeRef, RustTypeRef},
    CopyableToCArray,
};
decl_c_type!(loaned(z_clock_t, Instant));

/// Returns monotonic clock time point corresponding to the current time instant.
#[no_mangle]
pub extern "C" fn z_clock_now() -> z_clock_t {
    *Instant::now().as_loaned_c_type_ref()
}

/// Get number of nanoseconds passed since creation of `time`.
#[allow(clippy::missing_safety_doc)]
unsafe fn get_elapsed_nanos(time: &z_clock_t) -> u64 {
    time.as_rust_type_ref().elapsed().as_nanos() as u64
}

/// Get number of seconds passed since creation of `time`.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn z_clock_elapsed_s(time: &z_clock_t) -> u64 {
    get_elapsed_nanos(time) / 1_000_000_000
}

/// Get number of milliseconds passed since creation of `time`.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn z_clock_elapsed_ms(time: &z_clock_t) -> u64 {
    get_elapsed_nanos(time) / 1_000_000
}

/// Get number of microseconds passed since creation of `time`.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn z_clock_elapsed_us(time: &z_clock_t) -> u64 {
    get_elapsed_nanos(time) / 1_000
}

pub use crate::z_time_t;
decl_c_type!(loaned(z_time_t, SystemTime));

/// Converts current system time into null-terminated human readable string and writes it to the `buf`.
///
/// @param buf: A buffer where the string will be writtent
/// @param len: Maximum number of characters to write (including terminating 0). The string will be truncated
/// if it is longer than `len`.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn z_time_now_as_str(buf: *const c_char, len: usize) -> *const c_char {
    if len == 0 || buf.is_null() {
        return buf;
    }
    let datetime: DateTime<Local> = SystemTime::now().into();
    let s = datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let res = s.as_str().copy_to_c_array(buf as *mut c_void, len - 1);
    *((buf as usize + res) as *mut c_char) = 0;
    buf
}

/// Initialize clock with current time instant.
#[no_mangle]
pub extern "C" fn z_time_now() -> z_time_t {
    *SystemTime::now().as_loaned_c_type_ref()
}

/// Get number of nanoseconds passed since creation of `time`.
#[allow(clippy::missing_safety_doc)]
unsafe fn get_elapsed_nanos_system_clock(time: &z_time_t) -> u64 {
    match time.as_rust_type_ref().elapsed() {
        Ok(d) => d.as_nanos() as u64,
        Err(_) => 0,
    }
}

/// Get number of seconds passed since creation of `time`.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn z_time_elapsed_s(time: &z_time_t) -> u64 {
    get_elapsed_nanos_system_clock(time) / 1_000_000_000
}

/// Get number of milliseconds passed since creation of `time`.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn z_time_elapsed_ms(time: &z_time_t) -> u64 {
    get_elapsed_nanos_system_clock(time) / 1_000_000
}

/// Get number of microseconds passed since creation of `time`.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn z_time_elapsed_us(time: &z_time_t) -> u64 {
    get_elapsed_nanos_system_clock(time) / 1_000
}
