use crate::cell::UnsafeCell;
use crate::mem::MaybeUninit;
use crate::sys::c;

/// Mutex based on critical sections.
///
/// Critical sections are available on all windows versions, but `TryEnterCriticalSection` was only
/// added with NT4, and never to the 9x range.
///
/// It cannot be directly `const`-created as it needs to be initialized, and cannot be moved after
/// initialization. The top-level `Mutex` type handles boxing.
pub struct CriticalSectionMutex {
    inner: MaybeUninit<UnsafeCell<c::CRITICAL_SECTION>>,
}

unsafe impl Send for CriticalSectionMutex {}
unsafe impl Sync for CriticalSectionMutex {}

impl CriticalSectionMutex {
    #[inline]
    pub const fn new() -> Self {
        Self { inner: MaybeUninit::uninit() }
    }

    #[inline]
    pub unsafe fn init(&self) {
        c::InitializeCriticalSection(UnsafeCell::raw_get(self.inner.as_ptr()));
    }

    #[inline]
    pub unsafe fn lock(&self) {
        c::EnterCriticalSection(UnsafeCell::raw_get(self.inner.as_ptr()));
    }

    #[inline]
    pub unsafe fn try_lock(&self) -> bool {
        c::TryEnterCriticalSection::call(UnsafeCell::raw_get(self.inner.as_ptr())) != 0
    }

    #[inline]
    pub unsafe fn unlock(&self) {
        c::LeaveCriticalSection(UnsafeCell::raw_get(self.inner.as_ptr()));
    }

    #[inline]
    pub unsafe fn destroy(&self) -> bool {
        if self.try_lock() {
            self.unlock();
            c::DeleteCriticalSection(UnsafeCell::raw_get(self.inner.as_ptr()));
            true
        } else {
            // mutex is still locked, cannot destroy. caller needs to leak it instead
            false
        }
    }
}
