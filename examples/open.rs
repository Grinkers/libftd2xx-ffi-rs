use std::ffi::CString;
use std::os::raw::c_void;

use libftd2xx_ffi::{FT_OpenEx, FT_HANDLE, FT_OPEN_BY_DESCRIPTION};

fn main() {
    unsafe {
        let io_ident = CString::new("USB INTERFACE").expect("CString::new failed");
        let mut ft_handle: FT_HANDLE = std::ptr::null_mut();

        FT_OpenEx(
            io_ident.as_ptr() as *mut c_void,
            FT_OPEN_BY_DESCRIPTION,
            &mut ft_handle,
        );
    }
}
