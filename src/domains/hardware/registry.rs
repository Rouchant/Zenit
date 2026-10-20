use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, KEY_READ,
};

pub fn read_reg_string(hkey: HKEY, subkey: &str, value_name: &str) -> Option<String> {
    let subkey_utf16: Vec<u16> = subkey.encode_utf16().chain(std::iter::once(0)).collect();
    let value_name_utf16: Vec<u16> = value_name.encode_utf16().chain(std::iter::once(0)).collect();
    let mut opened_key: HKEY = std::ptr::null_mut();

    unsafe {
        if RegOpenKeyExW(
            hkey,
            subkey_utf16.as_ptr(),
            0,
            KEY_READ,
            &mut opened_key,
        ) != 0 {
            return None;
        }

        let mut data_type: u32 = 0;
        let mut data_size: u32 = 0;

        // First call to get size
        let status = RegQueryValueExW(
            opened_key,
            value_name_utf16.as_ptr(),
            std::ptr::null_mut(),
            &mut data_type,
            std::ptr::null_mut(),
            &mut data_size,
        );

        if status != 0 || data_size == 0 {
            RegCloseKey(opened_key);
            return None;
        }

        let mut buffer: Vec<u8> = vec![0; data_size as usize];
        let status = RegQueryValueExW(
            opened_key,
            value_name_utf16.as_ptr(),
            std::ptr::null_mut(),
            &mut data_type,
            buffer.as_mut_ptr(),
            &mut data_size,
        );

        RegCloseKey(opened_key);

        if status == 0 {
            let u16_slice = std::slice::from_raw_parts(
                buffer.as_ptr() as *const u16,
                (data_size as usize) / 2,
            );
            // Trim trailing nulls
            let trimmed: Vec<u16> = u16_slice.iter().copied().take_while(|&c| c != 0).collect();
            let os_str = OsString::from_wide(&trimmed);
            return os_str.into_string().ok().map(|s| s.trim().to_string());
        }
    }
    None
}
