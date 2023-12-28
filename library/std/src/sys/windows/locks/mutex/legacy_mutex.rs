use crate::io;
use crate::ptr;
use crate::sys::{c, cvt};

/// Mutex based on `CreateMutex`. Slow, but available everywhere.
///
/// Doesn't need to stay fixed in place, so it doesn't need to be boxed.
#[repr(transparent)]
pub struct LegacyMutex(c::HANDLE);

unsafe impl Send for LegacyMutex {}
unsafe impl Sync for LegacyMutex {}

impl LegacyMutex {
    #[inline]
    pub unsafe fn new() -> Self {
        let handle = c::CreateMutexA(ptr::null_mut(), c::FALSE, ptr::null());

        if handle.is_null() {
            panic!("failed creating mutex: {}", io::Error::last_os_error());
        }
        Self(handle)
    }

    #[inline]
    pub unsafe fn lock(&self) {
        if c::WaitForSingleObject(self.0, c::INFINITE) != c::WAIT_OBJECT_0 {
            panic!("mutex lock failed: {}", io::Error::last_os_error())
        }
    }

    #[inline]
    pub unsafe fn try_lock(&self) -> bool {
        match c::WaitForSingleObject(self.0, 0) {
            c::WAIT_OBJECT_0 => true,
            c::WAIT_TIMEOUT => false,
            _ => panic!("try lock error: {}", io::Error::last_os_error()),
        }
    }

    #[inline]
    pub unsafe fn unlock(&self) {
        cvt(c::ReleaseMutex(self.0)).unwrap();
    }
}

impl Drop for LegacyMutex {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            cvt(c::CloseHandle(self.0)).unwrap();
        }
    }
}
