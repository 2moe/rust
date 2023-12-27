#![cfg_attr(test, allow(dead_code))]

use crate::sys::c;
use crate::thread;

use super::api;

pub struct Handler;

impl Handler {
    pub unsafe fn new() -> Handler {
        if let Some(f) = c::SetThreadStackGuarantee::option() {
            if f(&mut 0x5000) == 0 && api::get_last_error().code != c::ERROR_CALL_NOT_IMPLEMENTED {
                panic!("failed to reserve stack space for exception handling");
            }
        };

        Handler
    }
}

unsafe extern "system" fn vectored_handler(ExceptionInfo: *mut c::EXCEPTION_POINTERS) -> c::LONG {
    unsafe {
        let rec = &(*(*ExceptionInfo).ExceptionRecord);
        let code = rec.ExceptionCode;

        if code == c::EXCEPTION_STACK_OVERFLOW {
            rtprintpanic!(
                "\nthread '{}' has overflowed its stack\n",
                thread::current().name().unwrap_or("<unknown>")
            );
        }
        c::EXCEPTION_CONTINUE_SEARCH
    }
}

pub unsafe fn init() {
    let Some(f) = c::AddVectoredExceptionHandler::option() else {
        return;
    };

    if f(0, Some(vectored_handler)).is_null() {
        panic!("failed to install exception handler");
    }
    // Set the thread stack guarantee for the main thread.
    let _h = Handler::new();
}
