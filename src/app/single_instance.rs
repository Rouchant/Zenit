#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE, HWND};
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE, SW_SHOW,
};

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateMutexW(
        lpMutexAttributes: *const std::ffi::c_void,
        bInitialOwner: i32,
        lpName: *const u16,
    ) -> HANDLE;
}

pub struct SingleInstanceGuard {
    #[cfg(target_os = "windows")]
    handle: HANDLE,
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        if !self.handle.is_null() {
            unsafe {
                CloseHandle(self.handle);
            }
        }
    }
}

pub fn acquire_single_instance() -> Option<SingleInstanceGuard> {
    #[cfg(target_os = "windows")]
    unsafe {
        let name: Vec<u16> = "Local\\ZenitShowroomSingleInstanceMutex\0"
            .encode_utf16()
            .collect();

        let handle = CreateMutexW(std::ptr::null(), 1, name.as_ptr());
        if handle.is_null() {
            return Some(SingleInstanceGuard { handle });
        }

        if GetLastError() == ERROR_ALREADY_EXISTS {
            println!("[Zenit] Ya existe una instancia de Zenit Showroom en ejecución. Restaurando ventana activa...");

            let title: Vec<u16> = "Showroom PC - Modo Exhibición\0"
                .encode_utf16()
                .collect();
            let hwnd: HWND = FindWindowW(std::ptr::null(), title.as_ptr());
            if !hwnd.is_null() {
                ShowWindow(hwnd, SW_SHOW);
                ShowWindow(hwnd, SW_RESTORE);
                SetForegroundWindow(hwnd);
            }

            CloseHandle(handle);
            return None;
        }

        Some(SingleInstanceGuard { handle })
    }

    #[cfg(not(target_os = "windows"))]
    {
        Some(SingleInstanceGuard {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_instance_guard_lifecycle() {
        let guard1 = acquire_single_instance();
        assert!(guard1.is_some(), "First instance should acquire guard");

        let guard2 = acquire_single_instance();
        assert!(guard2.is_none(), "Second instance should be rejected");

        drop(guard1);

        let guard3 = acquire_single_instance();
        assert!(guard3.is_some(), "After dropping first instance, next should acquire guard");
    }
}
