use super::{
    compat::{MutexKind, MUTEX_KIND},
    Mutex,
};
use crate::ops::Deref;

pub struct RwLock {
    pub(super) inner: Mutex,
}

unsafe impl Send for RwLock {}
unsafe impl Sync for RwLock {}

impl RwLock {
    #[inline]
    pub const fn new() -> RwLock {
        RwLock { inner: Mutex::new() }
    }
    #[inline]
    pub unsafe fn read(&self) {
        match MUTEX_KIND {
            MutexKind::SrwLock => self.inner.inner.deref().srwlock.read(),
            MutexKind::CriticalSection | MutexKind::Legacy => self.inner.lock(),
        }
    }
    #[inline]
    pub unsafe fn try_read(&self) -> bool {
        match MUTEX_KIND {
            MutexKind::SrwLock => self.inner.inner.deref().srwlock.try_read(),
            MutexKind::CriticalSection | MutexKind::Legacy => self.inner.try_lock(),
        }
    }
    #[inline]
    pub unsafe fn write(&self) {
        match MUTEX_KIND {
            MutexKind::SrwLock => self.inner.inner.deref().srwlock.write(),
            MutexKind::CriticalSection | MutexKind::Legacy => self.inner.lock(),
        }
    }
    #[inline]
    pub unsafe fn try_write(&self) -> bool {
        match MUTEX_KIND {
            MutexKind::SrwLock => self.inner.inner.deref().srwlock.try_write(),
            MutexKind::CriticalSection | MutexKind::Legacy => self.inner.try_lock(),
        }
    }
    #[inline]
    pub unsafe fn read_unlock(&self) {
        match MUTEX_KIND {
            MutexKind::SrwLock => self.inner.inner.deref().srwlock.read_unlock(),
            MutexKind::CriticalSection | MutexKind::Legacy => self.inner.unlock(),
        }
    }
    #[inline]
    pub unsafe fn write_unlock(&self) {
        match MUTEX_KIND {
            MutexKind::SrwLock => self.inner.inner.deref().srwlock.write_unlock(),
            MutexKind::CriticalSection | MutexKind::Legacy => self.inner.unlock(),
        }
    }
}
