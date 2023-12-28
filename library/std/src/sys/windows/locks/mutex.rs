//! System Mutexes
//!
//! The Windows implementation of mutexes is a little odd and it might not be
//! immediately obvious what's going on. The primary oddness is that SRWLock is
//! used instead of CriticalSection, and this is done because:
//!
//! 1. SRWLock is several times faster than CriticalSection according to
//!    benchmarks performed on both Windows 8 and Windows 7.
//!
//! 2. CriticalSection allows recursive locking while SRWLock deadlocks. The
//!    Unix implementation deadlocks so consistency is preferred. See #19962 for
//!    more details.
//!
//! 3. While CriticalSection is fair and SRWLock is not, the current Rust policy
//!    is that there are no guarantees of fairness.

use self::compat::{MutexKind, MUTEX_KIND};
use crate::cell::UnsafeCell;
use crate::mem::ManuallyDrop;
use crate::ops::Deref;
use crate::sys_common::lazy_box::{LazyBox, LazyInit};

pub mod compat;
mod critical_section_mutex;
mod legacy_mutex;
mod srwlock;

pub union InnerMutex {
    pub(super) srwlock: ManuallyDrop<srwlock::SrwLock>,
    critical_section: ManuallyDrop<critical_section_mutex::CriticalSectionMutex>,
    legacy: ManuallyDrop<legacy_mutex::LegacyMutex>,
}

impl Drop for InnerMutex {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            match MUTEX_KIND {
                MutexKind::SrwLock => ManuallyDrop::drop(&mut self.srwlock),
                MutexKind::CriticalSection => {
                    if self.critical_section.destroy() {
                        ManuallyDrop::drop(&mut self.critical_section);
                    } else {
                        // The mutex is locked. This happens if a MutexGuard is leaked.
                        // In this case, we just leak the Mutex too.
                    }
                }
                MutexKind::Legacy => ManuallyDrop::drop(&mut self.legacy),
            }
        }
    }
}

pub struct Mutex {
    pub inner: LazyBox<InnerMutex>,
    // used to prevent reentrancy for critical sections and legacy mutexes:
    //
    // > The exact behavior on locking a mutex in the thread which already holds the lock is left
    // > unspecified. However, this function will not return on the second call (it might panic or
    // > deadlock, for example).
    held: UnsafeCell<bool>,
}

unsafe impl Send for Mutex {}
unsafe impl Sync for Mutex {}

impl Mutex {
    #[inline]
    pub const fn new() -> Mutex {
        Mutex { inner: LazyBox::new(), held: UnsafeCell::new(false) }
    }

    #[inline]
    pub fn lock(&self) {
        let m = self.inner.deref();

        unsafe {
            match MUTEX_KIND {
                MutexKind::SrwLock => m.srwlock.write(),
                MutexKind::CriticalSection => {
                    m.critical_section.lock();
                    if !self.flag_locked() {
                        self.unlock();
                        panic!("cannot recursively lock a mutex");
                    }
                }
                MutexKind::Legacy => {
                    m.legacy.lock();
                    if !self.flag_locked() {
                        self.unlock();
                        panic!("cannot recursively lock a mutex");
                    }
                }
            }
        }
    }

    #[inline]
    pub fn try_lock(&self) -> bool {
        let m = self.inner.deref();

        unsafe {
            match MUTEX_KIND {
                MutexKind::SrwLock => m.srwlock.try_write(),
                MutexKind::CriticalSection => {
                    if !m.critical_section.try_lock() {
                        false
                    } else if self.flag_locked() {
                        true
                    } else {
                        self.unlock();
                        false
                    }
                }
                MutexKind::Legacy => {
                    if !m.legacy.try_lock() {
                        false
                    } else if self.flag_locked() {
                        true
                    } else {
                        self.unlock();
                        false
                    }
                }
            }
        }
    }

    #[inline]
    pub unsafe fn unlock(&self) {
        let m = self.inner.deref();

        unsafe {
            match MUTEX_KIND {
                MutexKind::SrwLock => m.srwlock.write_unlock(),
                MutexKind::CriticalSection => {
                    *self.held.get() = false;
                    m.critical_section.unlock();
                }
                MutexKind::Legacy => {
                    *self.held.get() = false;
                    m.legacy.unlock();
                }
            }
        }
    }

    unsafe fn flag_locked(&self) -> bool {
        if *self.held.get() {
            false
        } else {
            *self.held.get() = true;
            true
        }
    }
}

impl LazyInit for InnerMutex {
    fn init() -> Box<Self> {
        unsafe {
            match MUTEX_KIND {
                MutexKind::SrwLock => {
                    Box::new(InnerMutex { srwlock: ManuallyDrop::new(srwlock::SrwLock::new()) })
                }
                MutexKind::CriticalSection => {
                    let boxed = Box::new(InnerMutex {
                        critical_section: ManuallyDrop::new(
                            critical_section_mutex::CriticalSectionMutex::new(),
                        ),
                    });
                    boxed.critical_section.init();

                    boxed
                }
                MutexKind::Legacy => Box::new(InnerMutex {
                    legacy: ManuallyDrop::new(legacy_mutex::LegacyMutex::new()),
                }),
            }
        }
    }

    fn cancel_init(_: Box<Self>) {}
    fn destroy(_: Box<Self>) {}
}
