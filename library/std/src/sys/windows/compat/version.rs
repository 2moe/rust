use crate::sys::c;

static mut IS_NT: bool = true;
static mut SUPPORTS_ASYNC_IO: bool = true;

pub fn init_windows_version_check() {
    // according to old MSDN info, the high-order bit is set only on 95/98/ME.
    unsafe {
        IS_NT = c::GetVersion() < 0x8000_0000;
        SUPPORTS_ASYNC_IO = IS_NT && c::CancelIo::option().is_some();
    };
}

/// Returns true if we are running on a Windows NT-based system. Only use this for APIs where the
/// same API differs in behavior or capability on 9x/ME compared to NT.
#[inline(always)]
pub fn is_windows_nt() -> bool {
    unsafe { IS_NT }
}

#[inline(always)]
pub fn supports_async_io() -> bool {
    unsafe { SUPPORTS_ASYNC_IO }
}
