//! C definitions used by libnative that don't belong in liblibc

#![allow(nonstandard_style)]
#![cfg_attr(test, allow(dead_code))]
#![unstable(issue = "none", feature = "windows_c")]
#![allow(clippy::style)]

use crate::mem;
pub use crate::os::raw::c_int;
use crate::os::raw::{c_char, c_long, c_longlong, c_uint, c_ulong, c_ushort, c_void};
use crate::os::windows::io::{AsRawHandle, BorrowedHandle};
use crate::ptr;
use core::ffi::NonZero_c_ulong;

mod windows_sys;
mod wspiapi;
pub use windows_sys::*;

pub type DWORD = c_ulong;
pub type NonZeroDWORD = NonZero_c_ulong;
pub type LARGE_INTEGER = c_longlong;
#[cfg_attr(target_vendor = "uwp", allow(unused))]
pub type LONG = c_long;
pub type UINT = c_uint;
pub type WCHAR = u16;
pub type USHORT = c_ushort;
pub type SIZE_T = usize;
pub type WORD = u16;
pub type CHAR = c_char;
pub type ULONG = c_ulong;
pub type ACCESS_MASK = DWORD;

pub type LPCVOID = *const c_void;
pub type LPHANDLE = *mut HANDLE;
pub type LPOVERLAPPED = *mut OVERLAPPED;
pub type LPSECURITY_ATTRIBUTES = *mut SECURITY_ATTRIBUTES;
pub type LPVOID = *mut c_void;
pub type LPWCH = *mut WCHAR;
pub type LPWSTR = *mut WCHAR;

pub type PLARGE_INTEGER = *mut c_longlong;

pub type socklen_t = c_int;
pub type ADDRESS_FAMILY = USHORT;
pub use FD_SET as fd_set;
pub use LINGER as linger;
pub use TIMEVAL as timeval;

pub const INVALID_HANDLE_VALUE: HANDLE = ::core::ptr::invalid_mut(-1i32 as _);

// https://learn.microsoft.com/en-us/cpp/c-runtime-library/exit-success-exit-failure?view=msvc-170
pub const EXIT_SUCCESS: u32 = 0;
pub const EXIT_FAILURE: u32 = 1;

pub const CONDITION_VARIABLE_INIT: CONDITION_VARIABLE = CONDITION_VARIABLE { Ptr: ptr::null_mut() };
pub const SRWLOCK_INIT: SRWLOCK = SRWLOCK { Ptr: ptr::null_mut() };

// Some windows_sys types have different signs than the types we use.
pub const OBJ_DONT_REPARSE: u32 = windows_sys::OBJ_DONT_REPARSE as u32;
pub const FRS_ERR_SYSVOL_POPULATE_TIMEOUT: u32 =
    windows_sys::FRS_ERR_SYSVOL_POPULATE_TIMEOUT as u32;
pub const AF_INET: c_int = windows_sys::AF_INET as c_int;
pub const AF_INET6: c_int = windows_sys::AF_INET6 as c_int;

#[repr(C)]
pub struct ip_mreq {
    pub imr_multiaddr: in_addr,
    pub imr_interface: in_addr,
}

#[repr(C)]
pub struct ipv6_mreq {
    pub ipv6mr_multiaddr: in6_addr,
    pub ipv6mr_interface: c_uint,
}

// Equivalent to the `NT_SUCCESS` C preprocessor macro.
// See: https://docs.microsoft.com/en-us/windows-hardware/drivers/kernel/using-ntstatus-values
pub fn nt_success(status: NTSTATUS) -> bool {
    status >= 0
}

impl UNICODE_STRING {
    pub fn from_ref(slice: &[u16]) -> Self {
        let len = mem::size_of_val(slice);
        Self { Length: len as _, MaximumLength: len as _, Buffer: slice.as_ptr() as _ }
    }
}

impl Default for OBJECT_ATTRIBUTES {
    fn default() -> Self {
        Self {
            Length: mem::size_of::<Self>() as _,
            RootDirectory: ptr::null_mut(),
            ObjectName: ptr::null_mut(),
            Attributes: 0,
            SecurityDescriptor: ptr::null_mut(),
            SecurityQualityOfService: ptr::null_mut(),
        }
    }
}

impl IO_STATUS_BLOCK {
    pub const PENDING: Self =
        IO_STATUS_BLOCK { Anonymous: IO_STATUS_BLOCK_0 { Status: STATUS_PENDING }, Information: 0 };
    pub fn status(&self) -> NTSTATUS {
        // SAFETY: If `self.Anonymous.Status` was set then this is obviously safe.
        // If `self.Anonymous.Pointer` was set then this is the equivalent to converting
        // the pointer to an integer, which is also safe.
        // Currently the only safe way to construct `IO_STATUS_BLOCK` outside of
        // this module is to call the `default` method, which sets the `Status`.
        unsafe { self.Anonymous.Status }
    }
}

/// NB: Use carefully! In general using this as a reference is likely to get the
/// provenance wrong for the `rest` field!
#[repr(C)]
pub struct REPARSE_DATA_BUFFER {
    pub ReparseTag: c_uint,
    pub ReparseDataLength: c_ushort,
    pub Reserved: c_ushort,
    pub rest: (),
}

/// NB: Use carefully! In general using this as a reference is likely to get the
/// provenance wrong for the `PathBuffer` field!
#[repr(C)]
pub struct SYMBOLIC_LINK_REPARSE_BUFFER {
    pub SubstituteNameOffset: c_ushort,
    pub SubstituteNameLength: c_ushort,
    pub PrintNameOffset: c_ushort,
    pub PrintNameLength: c_ushort,
    pub Flags: c_ulong,
    pub PathBuffer: WCHAR,
}

#[repr(C)]
pub struct MOUNT_POINT_REPARSE_BUFFER {
    pub SubstituteNameOffset: c_ushort,
    pub SubstituteNameLength: c_ushort,
    pub PrintNameOffset: c_ushort,
    pub PrintNameLength: c_ushort,
    pub PathBuffer: WCHAR,
}
#[repr(C)]
pub struct REPARSE_MOUNTPOINT_DATA_BUFFER {
    pub ReparseTag: DWORD,
    pub ReparseDataLength: DWORD,
    pub Reserved: WORD,
    pub ReparseTargetLength: WORD,
    pub ReparseTargetMaximumLength: WORD,
    pub Reserved1: WORD,
    pub ReparseTarget: WCHAR,
}

#[repr(C)]
pub struct SOCKADDR_STORAGE_LH {
    pub ss_family: ADDRESS_FAMILY,
    pub __ss_pad1: [CHAR; 6],
    pub __ss_align: i64,
    pub __ss_pad2: [CHAR; 112],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct sockaddr_in {
    pub sin_family: ADDRESS_FAMILY,
    pub sin_port: USHORT,
    pub sin_addr: in_addr,
    pub sin_zero: [CHAR; 8],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct sockaddr_in6 {
    pub sin6_family: ADDRESS_FAMILY,
    pub sin6_port: USHORT,
    pub sin6_flowinfo: c_ulong,
    pub sin6_addr: in6_addr,
    pub sin6_scope_id: c_ulong,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct in_addr {
    pub s_addr: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct in6_addr {
    pub s6_addr: [u8; 16],
}

// Desktop specific functions & types
cfg_if::cfg_if! {
if #[cfg(not(target_vendor = "uwp"))] {
    pub const EXCEPTION_CONTINUE_SEARCH: i32 = 0;
}
}

pub unsafe extern "system" fn WriteFileEx(
    hFile: BorrowedHandle<'_>,
    lpBuffer: *mut ::core::ffi::c_void,
    nNumberOfBytesToWrite: u32,
    lpOverlapped: *mut OVERLAPPED,
    lpCompletionRoutine: LPOVERLAPPED_COMPLETION_ROUTINE,
) -> BOOL {
    windows_sys::WriteFileEx(
        hFile.as_raw_handle(),
        lpBuffer.cast::<u8>(),
        nNumberOfBytesToWrite,
        lpOverlapped,
        lpCompletionRoutine,
    )
}

pub unsafe extern "system" fn ReadFileEx(
    hFile: BorrowedHandle<'_>,
    lpBuffer: *mut ::core::ffi::c_void,
    nNumberOfBytesToRead: u32,
    lpOverlapped: *mut OVERLAPPED,
    lpCompletionRoutine: LPOVERLAPPED_COMPLETION_ROUTINE,
) -> BOOL {
    windows_sys::ReadFileEx(
        hFile.as_raw_handle(),
        lpBuffer.cast::<u8>(),
        nNumberOfBytesToRead,
        lpOverlapped,
        lpCompletionRoutine,
    )
}

// POSIX compatibility shims.
pub unsafe fn recv(socket: SOCKET, buf: *mut c_void, len: c_int, flags: c_int) -> c_int {
    windows_sys::recv(socket, buf.cast::<u8>(), len, flags)
}
pub unsafe fn send(socket: SOCKET, buf: *const c_void, len: c_int, flags: c_int) -> c_int {
    windows_sys::send(socket, buf.cast::<u8>(), len, flags)
}
pub unsafe fn recvfrom(
    socket: SOCKET,
    buf: *mut c_void,
    len: c_int,
    flags: c_int,
    addr: *mut SOCKADDR,
    addrlen: *mut c_int,
) -> c_int {
    windows_sys::recvfrom(socket, buf.cast::<u8>(), len, flags, addr, addrlen)
}
pub unsafe fn sendto(
    socket: SOCKET,
    buf: *const c_void,
    len: c_int,
    flags: c_int,
    addr: *const SOCKADDR,
    addrlen: c_int,
) -> c_int {
    windows_sys::sendto(socket, buf.cast::<u8>(), len, flags, addr, addrlen)
}
pub unsafe fn getaddrinfo(
    node: *const c_char,
    service: *const c_char,
    hints: *const ADDRINFOA,
    res: *mut *mut ADDRINFOA,
) -> c_int {
    ws2_32::getaddrinfo(node.cast::<u8>(), service.cast::<u8>(), hints, res)
}

cfg_if::cfg_if! {
if #[cfg(not(target_vendor = "uwp"))] {
pub unsafe fn NtReadFile(
    filehandle: BorrowedHandle<'_>,
    event: HANDLE,
    apcroutine: PIO_APC_ROUTINE,
    apccontext: *mut c_void,
    iostatusblock: &mut IO_STATUS_BLOCK,
    buffer: *mut crate::mem::MaybeUninit<u8>,
    length: ULONG,
    byteoffset: Option<&LARGE_INTEGER>,
    key: Option<&ULONG>,
) -> NTSTATUS {
    ntdll::NtReadFile(
        filehandle.as_raw_handle(),
        event,
        apcroutine,
        apccontext,
        iostatusblock,
        buffer.cast::<c_void>(),
        length,
        byteoffset.map(|o| o as *const i64).unwrap_or(ptr::null()),
        key.map(|k| k as *const u32).unwrap_or(ptr::null()),
    )
}
pub unsafe fn NtWriteFile(
    filehandle: BorrowedHandle<'_>,
    event: HANDLE,
    apcroutine: PIO_APC_ROUTINE,
    apccontext: *mut c_void,
    iostatusblock: &mut IO_STATUS_BLOCK,
    buffer: *const u8,
    length: ULONG,
    byteoffset: Option<&LARGE_INTEGER>,
    key: Option<&ULONG>,
) -> NTSTATUS {
    ntdll::NtWriteFile(
        filehandle.as_raw_handle(),
        event,
        apcroutine,
        apccontext,
        iostatusblock,
        buffer.cast::<c_void>(),
        length,
        byteoffset.map(|o| o as *const i64).unwrap_or(ptr::null()),
        key.map(|k| k as *const u32).unwrap_or(ptr::null()),
    )
}
}
}

// Functions that aren't available on every version of Windows that we support,
// but we still use them and just provide some form of a fallback implementation.
compat_fn_with_fallback! {
    pub static KERNEL32: &CStr = c"kernel32" => { load: false, unicows: false };

    // >= Win10 1607
    // https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-setthreaddescription
    pub fn SetThreadDescription(hthread: HANDLE, lpthreaddescription: PCWSTR) -> HRESULT {
        SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD); E_NOTIMPL
    }

    // >= Win8 / Server 2012
    // https://docs.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getsystemtimepreciseasfiletime
    pub fn GetSystemTimePreciseAsFileTime(lpsystemtimeasfiletime: *mut FILETIME) -> () {
        GetSystemTimeAsFileTime(lpsystemtimeasfiletime)
    }

    // >= Win11 / Server 2022
    // https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-gettemppath2a
    pub fn GetTempPath2W(bufferlength: u32, buffer: PWSTR) -> u32 {
        GetTempPathW(bufferlength, buffer)
    }

    // >= 95 / NT 3.5
    // https://docs.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getsystemtimeasfiletime
    pub fn GetSystemTimeAsFileTime(lpSystemTimeAsFileTime: *mut FILETIME) -> () {
        // implementation based on old MSDN docs
        let mut st: SYSTEMTIME = crate::mem::zeroed();
        GetSystemTime(&mut st);
        crate::sys::cvt(SystemTimeToFileTime(&st, lpSystemTimeAsFileTime)).unwrap();
    }

    // >= 2000
    // https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-setfilepointerex
    pub fn SetFilePointerEx(
        hfile: HANDLE,
        lidistancetomove: i64,
        lpnewfilepointer: *mut i64,
        dwmovemethod: SET_FILE_POINTER_MOVE_METHOD,
    ) -> BOOL {
        let lDistanceToMove = lidistancetomove as i32;
        let mut distance_to_move_high = (lidistancetomove >> 32) as i32;

        let newPos_low = SetFilePointer(hfile, lDistanceToMove, &mut distance_to_move_high, dwmovemethod);

        // since (-1 as u32) could be a valid value for the lower 32 bits of the new file pointer
        // position, a call to GetLastError is needed to actually see if it failed
        if newPos_low == INVALID_SET_FILE_POINTER && GetLastError() != NO_ERROR {
            return FALSE;
        }

        if !lpnewfilepointer.is_null() {
            *lpnewfilepointer = (distance_to_move_high as i64) << 32 | (newPos_low as i64);
        }

        TRUE
    }

    // >= Vista / Server 2008
    // https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createsymboliclinkw
    pub fn CreateSymbolicLinkW(
        lpsymlinkfilename: PCWSTR,
        lptargetfilename: PCWSTR,
        dwflags: SYMBOLIC_LINK_FLAGS,
    ) -> BOOLEAN {
        SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
        0
    }

    // >= 2000
    // https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createhardlinkw
    pub fn CreateHardLinkW(
        lpfilename: PCWSTR,
        lpexistingfilename: PCWSTR,
        lpsecurityattributes: *const SECURITY_ATTRIBUTES,
    ) -> BOOL {
        SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
        FALSE
    }

    // >= NT 3.51+
    // https://docs.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-sethandleinformation
    pub fn SetHandleInformation(hobject: HANDLE, dwmask: u32, dwflags: HANDLE_FLAGS) -> BOOL {
        SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
        FALSE
    }

    // >= NT 4+
    // https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-switchtothread
    pub fn SwitchToThread() -> BOOL {
        // A value of zero causes the thread to relinquish the remainder of its time slice to any
        // other thread of equal priority that is ready to run. If there are no other threads of
        // equal priority ready to run, the function returns immediately, and the thread continues
        // execution.
        Sleep(0);
        TRUE
    }

    // >= NT 3.5+, 95+
    // https://docs.microsoft.com/en-us/windows/win32/api/processenv/nf-processenv-freeenvironmentstringsw
    pub fn FreeEnvironmentStringsW(penv: PCWSTR) -> BOOL {
        // just leak it on NT 3.1
        TRUE
    }

    // >= Vista / Server 2008
    // https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-createwaitabletimerexw
    pub fn CreateWaitableTimerExW(
        lptimerattributes: *const SECURITY_ATTRIBUTES,
        lptimername: PCWSTR,
        dwflags: u32,
        dwdesiredaccess: u32,
    ) -> HANDLE {
        ptr::null_mut()
    }
}

compat_fn_lazy! {
    pub static KERNEL32: &CStr = c"kernel32" => { load: false, unicows: false };
    // >= Vista / Server 2008 (XP / Server 2003 when linking a supported FileExtd.lib)
    // https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-setfileinformationbyhandle
    pub fn SetFileInformationByHandle(
        hfile: HANDLE,
        fileinformationclass: FILE_INFO_BY_HANDLE_CLASS,
        lpfileinformation: *const ::core::ffi::c_void,
        dwbuffersize: u32,
    ) -> BOOL;
    // >= Vista / Server 2008 (XP / Server 2003 when linking a supported FileExtd.lib)
    // https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getfileinformationbyhandleex
    pub fn GetFileInformationByHandleEx(
        hfile: HANDLE,
        fileinformationclass: FILE_INFO_BY_HANDLE_CLASS,
        lpfileinformation: *mut ::core::ffi::c_void,
        dwbuffersize: u32,
    ) -> BOOL;

    // >= Vista / Server 2008
    // https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfinalpathnamebyhandlew
    pub fn GetFinalPathNameByHandleW(
        hfile: HANDLE,
        lpszfilepath: PWSTR,
        cchfilepath: u32,
        dwflags: GETFINALPATHNAMEBYHANDLE_FLAGS,
    ) -> u32;

    // >= NT 4+; partial: 95+ (would be provided by unicows, but only returns "not implemented")
    // https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-copyfileexw
    pub fn CopyFileExW(
        lpexistingfilename: PCWSTR,
        lpnewfilename: PCWSTR,
        lpprogressroutine: LPPROGRESS_ROUTINE,
        lpdata: *const ::core::ffi::c_void,
        pbcancel: *mut BOOL,
        dwcopyflags: u32,
    ) -> BOOL;

    // >= Vista / Server 2008
    // https://docs.microsoft.com/en-us/windows/win32/api/stringapiset/nf-stringapiset-comparestringordinal
    pub fn CompareStringOrdinal(
        lpstring1: PCWSTR,
        cchcount1: i32,
        lpstring2: PCWSTR,
        cchcount2: i32,
        bignorecase: BOOL,
    ) -> COMPARESTRING_RESULT;

    // >= NT4+, 98+
    // https://learn.microsoft.com/en-us/windows/win32/fileio/cancelio
    pub fn CancelIo(hfile: HANDLE) -> BOOL;

    // >= Vista / Server 2008
    // https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-initializeprocthreadattributelist
    pub fn InitializeProcThreadAttributeList(
        lpattributelist: LPPROC_THREAD_ATTRIBUTE_LIST,
        dwattributecount: u32,
        dwflags: u32,
        lpsize: *mut usize,
    ) -> BOOL;

    // >= Vista / Server 2008
    // https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute
    pub fn UpdateProcThreadAttribute(
        lpattributelist: LPPROC_THREAD_ATTRIBUTE_LIST,
        dwflags: u32,
        attribute: usize,
        lpvalue: *const ::core::ffi::c_void,
        cbsize: usize,
        lppreviousvalue: *mut ::core::ffi::c_void,
        lpreturnsize: *const usize,
    ) -> BOOL;

    // >= Vista / Server 2008
    // https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-deleteprocthreadattributelist
    pub fn DeleteProcThreadAttributeList(lpattributelist: LPPROC_THREAD_ATTRIBUTE_LIST) -> ();
}

compat_fn_optional! {
    crate::sys::compat::load_synch_functions();
    pub fn WaitOnAddress(
        address: *const ::core::ffi::c_void,
        compareaddress: *const ::core::ffi::c_void,
        addresssize: usize,
        dwmilliseconds: u32
    ) -> BOOL;
    pub fn WakeByAddressSingle(address: *const ::core::ffi::c_void);
}

compat_fn_optional! {
    crate::sys::compat::load_try_enter_critical_section_function();
    // >= NT 4
    // https://docs.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-tryentercriticalsection
    pub fn TryEnterCriticalSection(lpcriticalsection: *mut CRITICAL_SECTION) -> BOOL;
}

compat_fn_optional! {
    crate::sys::compat::load_srw_functions();
    // >= Win7 / Server 2008 R2
    // https://docs.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-tryacquiresrwlockexclusive
    pub fn TryAcquireSRWLockExclusive(srwlock: *mut SRWLOCK) -> BOOLEAN;
    pub fn TryAcquireSRWLockShared(srwlock: *mut SRWLOCK) -> BOOLEAN;
    // >= Vista / Server 2008
    // https://docs.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-acquiresrwlockexclusive
    pub fn AcquireSRWLockExclusive(srwlock: *mut SRWLOCK) -> ();
    pub fn AcquireSRWLockShared(srwlock: *mut SRWLOCK) -> ();
    pub fn ReleaseSRWLockExclusive(srwlock: *mut SRWLOCK) -> ();
    pub fn ReleaseSRWLockShared(srwlock: *mut SRWLOCK) -> ();
    pub fn SleepConditionVariableSRW(
        conditionvariable: *mut CONDITION_VARIABLE,
        srwlock: *mut SRWLOCK,
        dwmilliseconds: u32,
        flags: u32,
    ) -> BOOL;
    pub fn WakeAllConditionVariable(conditionvariable: *mut CONDITION_VARIABLE) -> ();
    pub fn WakeConditionVariable(conditionvariable: *mut CONDITION_VARIABLE) -> ();
}

compat_fn_lazy! {
    pub static USERENV: &CStr = c"userenv" => { load: true, unicows: false };

    // >= NT4
    // https://learn.microsoft.com/en-us/windows/win32/api/userenv/nf-userenv-getuserprofiledirectoryw
    pub fn GetUserProfileDirectoryW(
        htoken: HANDLE,
        lpprofiledir: PWSTR,
        lpcchsize: *mut u32,
    ) -> BOOL;
}

compat_fn_with_fallback! {
    pub static BCRYPT: &CStr = c"bcrypt" => { load: true, unicows: false };

    // >= Vista / Server 2008
    // https://learn.microsoft.com/en-us/windows/win32/api/bcrypt/nf-bcrypt-bcryptgenrandom
    pub fn BCryptGenRandom(
        halgorithm: BCRYPT_ALG_HANDLE,
        pbbuffer: *mut u8,
        cbbuffer: u32,
        dwflags: BCRYPTGENRANDOM_FLAGS,
    ) -> NTSTATUS {
        if SystemFunction036(pbbuffer.cast(), cbbuffer) == TRUE as _ {
            0 // STATUS_SUCCESS
        } else {
            0xC0000001u32 as i32 // STATUS_UNSUCCESSFUL
        }
    }
}

compat_fn_lazy! {
    pub static ADVAPI32: &CStr = c"advapi32" => { load: true, unicows: false };

    // NT only
    // https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-openprocesstoken
    pub fn OpenProcessToken(
        processhandle: HANDLE,
        desiredaccess: TOKEN_ACCESS_MASK,
        tokenhandle: *mut HANDLE,
    ) -> BOOL;
}
compat_fn_with_fallback! {
    pub static ADVAPI32: &CStr = c"advapi32" => { load: true, unicows: false };

    // >= XP / Server 2003
    // https://learn.microsoft.com/en-us/windows/win32/api/ntsecapi/nf-ntsecapi-rtlgenrandom
    pub fn SystemFunction036(randombuffer: *mut ::core::ffi::c_void, randombufferlength: u32)
    -> BOOLEAN {
        0
    }
}
pub const RtlGenRandom: unsafe fn(
    randombuffer: *mut ::core::ffi::c_void,
    randombufferlength: u32,
) -> BOOLEAN = SystemFunction036;

compat_fn_lazy! {
    pub static NTDLL: &CStr = c"ntdll" => { load: true, unicows: false };

    // NT only
    pub fn NtCreateFile(
        filehandle: *mut HANDLE,
        desiredaccess: FILE_ACCESS_RIGHTS,
        objectattributes: *const OBJECT_ATTRIBUTES,
        iostatusblock: *mut IO_STATUS_BLOCK,
        allocationsize: *const i64,
        fileattributes: FILE_FLAGS_AND_ATTRIBUTES,
        shareaccess: FILE_SHARE_MODE,
        createdisposition: NTCREATEFILE_CREATE_DISPOSITION,
        createoptions: NTCREATEFILE_CREATE_OPTIONS,
        eabuffer: *const ::core::ffi::c_void,
        ealength: u32,
    ) -> NTSTATUS;
}

pub mod ntdll {
    use super::*;
    compat_fn_lazy! {
        pub static NTDLL: &CStr = c"ntdll" => { load: true, unicows: false };

        // NT only
        pub fn NtReadFile(
            filehandle: HANDLE,
            event: HANDLE,
            apcroutine: PIO_APC_ROUTINE,
            apccontext: *const ::core::ffi::c_void,
            iostatusblock: *mut IO_STATUS_BLOCK,
            buffer: *mut ::core::ffi::c_void,
            length: u32,
            byteoffset: *const i64,
            key: *const u32,
        ) -> NTSTATUS;
        pub fn NtWriteFile(
            filehandle: HANDLE,
            event: HANDLE,
            apcroutine: PIO_APC_ROUTINE,
            apccontext: *const ::core::ffi::c_void,
            iostatusblock: *mut IO_STATUS_BLOCK,
            buffer: *const ::core::ffi::c_void,
            length: u32,
            byteoffset: *const i64,
            key: *const u32,
        ) -> NTSTATUS;
    }
}
compat_fn_with_fallback! {
    pub static NTDLL: &CStr = c"ntdll" => { load: true, unicows: false };

    pub fn NtCreateKeyedEvent(
        KeyedEventHandle: LPHANDLE,
        DesiredAccess: ACCESS_MASK,
        ObjectAttributes: LPVOID,
        Flags: ULONG
    ) -> NTSTATUS {
        panic!("keyed events not available")
    }
    pub fn NtReleaseKeyedEvent(
        EventHandle: HANDLE,
        Key: LPVOID,
        Alertable: BOOLEAN,
        Timeout: PLARGE_INTEGER
    ) -> NTSTATUS {
        panic!("keyed events not available")
    }
    pub fn NtWaitForKeyedEvent(
        EventHandle: HANDLE,
        Key: LPVOID,
        Alertable: BOOLEAN,
        Timeout: PLARGE_INTEGER
    ) -> NTSTATUS {
        panic!("keyed events not available")
    }
    pub fn RtlNtStatusToDosError(Status: NTSTATUS) -> u32 {
        Status as u32
    }
}

// # Arm32 shim
//
// AddVectoredExceptionHandler and WSAStartup use platform-specific types.
// However, Microsoft no longer supports thumbv7a so definitions for those targets
// are not included in the win32 metadata. We work around that by defining them here.
//
// Where possible, these definitions should be kept in sync with https://docs.rs/windows-sys
cfg_if::cfg_if! {
if #[cfg(not(target_vendor = "uwp"))] {
    compat_fn_optional! {
        crate::sys::compat::load_stack_overflow_functions();
        // >= Vista / Server 2003
        // https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-setthreadstackguarantee
        pub fn SetThreadStackGuarantee(stacksizeinbytes: *mut u32) -> BOOL;
        // >= XP
        // https://docs.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-addvectoredexceptionhandler
        pub fn AddVectoredExceptionHandler(
            first: u32,
            handler: PVECTORED_EXCEPTION_HANDLER,
        ) -> *mut c_void;
    }

    pub type PVECTORED_EXCEPTION_HANDLER = Option<
        unsafe extern "system" fn(exceptioninfo: *mut EXCEPTION_POINTERS) -> i32,
    >;
    #[repr(C)]
    pub struct EXCEPTION_POINTERS {
        pub ExceptionRecord: *mut EXCEPTION_RECORD,
        pub ContextRecord: *mut CONTEXT,
    }
    #[cfg(target_arch = "arm")]
    pub enum CONTEXT {}
}}

#[link(name = "ws2_32")]
extern "system" {
    pub fn WSAStartup(wversionrequested: u16, lpwsadata: *mut WSADATA) -> i32;
}
#[cfg(target_arch = "arm")]
#[repr(C)]
pub struct WSADATA {
    pub wVersion: u16,
    pub wHighVersion: u16,
    pub szDescription: [u8; 257],
    pub szSystemStatus: [u8; 129],
    pub iMaxSockets: u16,
    pub iMaxUdpDg: u16,
    pub lpVendorInfo: PSTR,
}

mod ws2_32 {
    use super::*;
    compat_fn_with_fallback! {
        pub static WS2_32: &CStr = c"ws2_32" => { load: true, unicows: false };

        // >= NT4/2000 with IPv6 Tech Preview
        pub fn getaddrinfo(
            pnodename: PCSTR,
            pservicename: PCSTR,
            phints: *const ADDRINFOA,
            ppresult: *mut *mut ADDRINFOA,
        ) -> i32 {
            wship6::getaddrinfo(pnodename, pservicename, phints, ppresult)
        }
        // >= NT4/2000 with IPv6 Tech Preview
        pub fn freeaddrinfo(paddrinfo: *const ADDRINFOA) -> () {
            wship6::freeaddrinfo(paddrinfo)
        }
    }
}
pub use ws2_32::freeaddrinfo;

mod wship6 {
    use super::wspiapi::{wspiapi_freeaddrinfo, wspiapi_getaddrinfo};
    use super::{ADDRINFOA, PCSTR};

    compat_fn_with_fallback! {
        pub static WSHIP6: &CStr = c"wship6" => { load: true, unicows: false };

        // >= 2000 with IPv6 Tech Preview
        pub fn getaddrinfo(
            pnodename: PCSTR,
            pservicename: PCSTR,
            phints: *const ADDRINFOA,
            ppresult: *mut *mut ADDRINFOA,
        ) -> i32 {
            wspiapi_getaddrinfo(pnodename, pservicename, phints, ppresult)
        }
        // >= 2000 with IPv6 Tech Preview
        pub fn freeaddrinfo(paddrinfo: *const ADDRINFOA)-> () {
            wspiapi_freeaddrinfo(paddrinfo)
        }
    }
}
