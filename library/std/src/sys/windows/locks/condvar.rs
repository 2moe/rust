use super::compat::{MutexKind, MUTEX_KIND};
use crate::cell::UnsafeCell;
use crate::io;
use crate::mem::ManuallyDrop;
use crate::ops::Deref;
use crate::ptr;
use crate::sys::c;
use crate::sys::cvt;
use crate::sys::locks::Mutex;
use crate::sys::os;
use crate::sys_common::lazy_box::{LazyBox, LazyInit};
use crate::time::Duration;

pub struct Condvar {
    inner: LazyBox<CondvarImpl>,
}

union CondvarImpl {
    srw: ManuallyDrop<UnsafeCell<c::CONDITION_VARIABLE>>,
    event: c::HANDLE,
}

impl Drop for CondvarImpl {
    fn drop(&mut self) {
        unsafe {
            match MUTEX_KIND {
                MutexKind::SrwLock => {}
                MutexKind::CriticalSection | MutexKind::Legacy => {
                    cvt(c::CloseHandle(self.event)).unwrap();
                }
            }
        }
    }
}

unsafe impl Send for Condvar {}
unsafe impl Sync for Condvar {}

impl Condvar {
    #[inline]
    pub const fn new() -> Condvar {
        Condvar { inner: LazyBox::new() }
    }

    #[inline]
    pub unsafe fn wait(&self, mutex: &Mutex) {
        let inner = self.inner.deref();

        match MUTEX_KIND {
            MutexKind::SrwLock => {
                let mutex = mutex.inner.deref();
                let r = c::SleepConditionVariableSRW(
                    inner.srw.get(),
                    mutex.srwlock.inner.get(),
                    c::INFINITE,
                    0,
                );
                debug_assert!(r != 0);
            }
            MutexKind::CriticalSection | MutexKind::Legacy => {
                mutex.unlock();
                if (c::WaitForSingleObject(inner.event, c::INFINITE)) != c::WAIT_OBJECT_0 {
                    panic!("event wait failed: {}", io::Error::last_os_error())
                }
                mutex.lock();
            }
        }
    }

    pub unsafe fn wait_timeout(&self, mutex: &Mutex, dur: Duration) -> bool {
        let inner = self.inner.deref();

        match MUTEX_KIND {
            MutexKind::SrwLock => {
                let mutex = mutex.inner.deref();
                let r = c::SleepConditionVariableSRW(
                    inner.srw.get(),
                    mutex.srwlock.inner.get(),
                    crate::sys::windows::dur2timeout(dur),
                    0,
                );
                if r == 0 {
                    debug_assert_eq!(os::errno() as usize, c::ERROR_TIMEOUT as usize);
                    false
                } else {
                    true
                }
            }
            MutexKind::CriticalSection | MutexKind::Legacy => {
                mutex.unlock();
                let ret = match c::WaitForSingleObject(
                    inner.event,
                    crate::sys::windows::dur2timeout(dur),
                ) {
                    c::WAIT_OBJECT_0 => true,
                    c::WAIT_TIMEOUT => false,
                    _ => panic!("event wait failed: {}", io::Error::last_os_error()),
                };
                mutex.lock();
                ret
            }
        }
    }

    #[inline]
    pub fn notify_one(&self) {
        let inner = self.inner.deref();

        unsafe {
            match MUTEX_KIND {
                MutexKind::SrwLock => c::WakeConditionVariable(inner.srw.get()),
                MutexKind::CriticalSection | MutexKind::Legacy => {
                    // this currently wakes up all threads, but spurious wakeups are allowed, so
                    // this is "just" reducing perf
                    cvt(c::PulseEvent(inner.event)).unwrap();
                }
            }
        }
    }

    #[inline]
    pub fn notify_all(&self) {
        let inner = self.inner.deref();

        unsafe {
            match MUTEX_KIND {
                MutexKind::SrwLock => c::WakeAllConditionVariable(inner.srw.get()),
                MutexKind::CriticalSection | MutexKind::Legacy => {
                    cvt(c::PulseEvent(inner.event)).unwrap();
                }
            }
        }
    }
}

impl LazyInit for CondvarImpl {
    fn init() -> Box<Self> {
        Box::new(unsafe {
            match MUTEX_KIND {
                MutexKind::SrwLock => CondvarImpl {
                    srw: ManuallyDrop::new(UnsafeCell::new(c::CONDITION_VARIABLE_INIT)),
                },
                MutexKind::CriticalSection | MutexKind::Legacy => {
                    let event = c::CreateEventA(
                        ptr::null_mut(),
                        c::TRUE, // manual reset event
                        c::FALSE,
                        ptr::null(),
                    );

                    if event.is_null() {
                        panic!("failed creating event: {}", io::Error::last_os_error());
                    }

                    CondvarImpl { event }
                }
            }
        })
    }

    fn cancel_init(_: Box<Self>) {}
    fn destroy(_: Box<Self>) {}
}
