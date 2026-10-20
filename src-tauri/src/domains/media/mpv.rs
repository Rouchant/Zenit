use libloading::{Library, Symbol};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
use std::ffi::{c_char, c_int, c_void, CString};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub const MPV_RENDER_PARAM_INVALID: u32 = 0;
pub const MPV_RENDER_PARAM_API_TYPE: u32 = 1;
pub const MPV_RENDER_PARAM_SW_SIZE: u32 = 7;
pub const MPV_RENDER_PARAM_SW_FORMAT: u32 = 8;
pub const MPV_RENDER_PARAM_SW_STRIDE: u32 = 9;
pub const MPV_RENDER_PARAM_SW_POINTER: u32 = 10;

pub const MPV_RENDER_API_TYPE_SW: &str = "sw";

#[repr(C)]
pub struct MpvRenderParam {
    pub param_type: u32,
    pub data: *mut c_void,
}

type MpvCreateFn = unsafe extern "C" fn() -> *mut c_void;
type MpvInitializeFn = unsafe extern "C" fn(*mut c_void) -> c_int;
type MpvDestroyFn = unsafe extern "C" fn(*mut c_void);
type MpvCommandStringFn = unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int;
type MpvSetOptionStringFn = unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> c_int;
type MpvRenderContextCreateFn = unsafe extern "C" fn(*mut *mut c_void, *mut c_void, *mut MpvRenderParam) -> c_int;
type MpvRenderContextRenderFn = unsafe extern "C" fn(*mut c_void, *mut MpvRenderParam) -> c_int;
type MpvRenderContextFreeFn = unsafe extern "C" fn(*mut c_void);

pub struct MpvLibrary {
    _lib: Library,
    pub mpv_create: MpvCreateFn,
    pub mpv_initialize: MpvInitializeFn,
    pub mpv_destroy: MpvDestroyFn,
    pub mpv_command_string: MpvCommandStringFn,
    pub mpv_set_option_string: MpvSetOptionStringFn,
    pub mpv_render_context_create: MpvRenderContextCreateFn,
    pub mpv_render_context_render: MpvRenderContextRenderFn,
    pub mpv_render_context_free: MpvRenderContextFreeFn,
}

impl MpvLibrary {
    pub fn load() -> Result<Self, String> {
        let candidates = [
            "mpv-2.dll",
            "libmpv-2.dll",
            "mpv-1.dll",
            "libmpv-1.dll",
            "mpv.dll",
        ];

        // Search relative to exe, cwd, and standard locations
        let search_dirs = [
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            std::env::current_exe().map(|p| p.parent().unwrap_or(Path::new(".")).to_path_buf()).unwrap_or_else(|_| PathBuf::from(".")),
            PathBuf::from("."),
            PathBuf::from("src-tauri"),
            PathBuf::from("target/debug"),
            PathBuf::from("target/release"),
        ];

        let mut library = None;

        for dir in &search_dirs {
            for name in &candidates {
                let candidate_path = dir.join(name);
                if candidate_path.exists() {
                    if let Ok(lib) = unsafe { Library::new(&candidate_path) } {
                        library = Some(lib);
                        break;
                    }
                }
            }
            if library.is_some() {
                break;
            }
        }

        // If not found in explicit paths, try loading by DLL name from system PATH
        if library.is_none() {
            for name in &candidates {
                if let Ok(lib) = unsafe { Library::new(name) } {
                    library = Some(lib);
                    break;
                }
            }
        }

        let lib = library.ok_or_else(|| {
            "libmpv no encontrado. Coloca mpv-2.dll o libmpv-2.dll en la carpeta de la aplicación.".to_string()
        })?;

        unsafe {
            let mpv_create: Symbol<MpvCreateFn> = lib.get(b"mpv_create\0").map_err(|e| e.to_string())?;
            let mpv_initialize: Symbol<MpvInitializeFn> = lib.get(b"mpv_initialize\0").map_err(|e| e.to_string())?;
            let mpv_destroy: Symbol<MpvDestroyFn> = lib.get(b"mpv_destroy\0").map_err(|e| e.to_string())?;
            let mpv_command_string: Symbol<MpvCommandStringFn> = lib.get(b"mpv_command_string\0").map_err(|e| e.to_string())?;
            let mpv_set_option_string: Symbol<MpvSetOptionStringFn> = lib.get(b"mpv_set_option_string\0").map_err(|e| e.to_string())?;
            let mpv_render_context_create: Symbol<MpvRenderContextCreateFn> = lib.get(b"mpv_render_context_create\0").map_err(|e| e.to_string())?;
            let mpv_render_context_render: Symbol<MpvRenderContextRenderFn> = lib.get(b"mpv_render_context_render\0").map_err(|e| e.to_string())?;
            let mpv_render_context_free: Symbol<MpvRenderContextFreeFn> = lib.get(b"mpv_render_context_free\0").map_err(|e| e.to_string())?;

            Ok(Self {
                mpv_create: *mpv_create,
                mpv_initialize: *mpv_initialize,
                mpv_destroy: *mpv_destroy,
                mpv_command_string: *mpv_command_string,
                mpv_set_option_string: *mpv_set_option_string,
                mpv_render_context_create: *mpv_render_context_create,
                mpv_render_context_render: *mpv_render_context_render,
                mpv_render_context_free: *mpv_render_context_free,
                _lib: lib,
            })
        }
    }
}

pub struct SlintMpvPlayer {
    mpv: *mut c_void,
    render_ctx: *mut c_void,
    lib: Arc<MpvLibrary>,
    _width: u32,
    _height: u32,
    size_param: [i32; 2],
    stride: usize,
    format_cstr: CString,
    _api_type_cstr: CString,
    pixel_buffer: SharedPixelBuffer<Rgba8Pixel>,
    is_playing: Arc<AtomicBool>,
}

unsafe impl Send for SlintMpvPlayer {}
unsafe impl Sync for SlintMpvPlayer {}

impl SlintMpvPlayer {
    pub fn new(video_path: &str, width: u32, height: u32) -> Result<Self, String> {
        let lib = Arc::new(MpvLibrary::load()?);

        unsafe {
            let mpv = (lib.mpv_create)();
            if mpv.is_null() {
                return Err("Fallo al crear instancia de mpv".to_string());
            }

            // Options for showroom playback
            let opt_loop = CString::new("loop-file").unwrap();
            let opt_inf = CString::new("inf").unwrap();
            (lib.mpv_set_option_string)(mpv, opt_loop.as_ptr(), opt_inf.as_ptr());

            let opt_hwdec = CString::new("hwdec").unwrap();
            let opt_auto = CString::new("auto").unwrap();
            (lib.mpv_set_option_string)(mpv, opt_hwdec.as_ptr(), opt_auto.as_ptr());

            let opt_vo = CString::new("vo").unwrap();
            let opt_null = CString::new("null").unwrap();
            (lib.mpv_set_option_string)(mpv, opt_vo.as_ptr(), opt_null.as_ptr());

            if (lib.mpv_initialize)(mpv) < 0 {
                (lib.mpv_destroy)(mpv);
                return Err("Fallo al inicializar mpv".to_string());
            }

            // Software Render Context
            let api_type_cstr = CString::new(MPV_RENDER_API_TYPE_SW).unwrap();
            let mut init_params = [
                MpvRenderParam {
                    param_type: MPV_RENDER_PARAM_API_TYPE,
                    data: api_type_cstr.as_ptr() as *mut c_void,
                },
                MpvRenderParam {
                    param_type: MPV_RENDER_PARAM_INVALID,
                    data: std::ptr::null_mut(),
                },
            ];

            let mut render_ctx: *mut c_void = std::ptr::null_mut();
            let res = (lib.mpv_render_context_create)(&mut render_ctx, mpv, init_params.as_mut_ptr());
            if res < 0 || render_ctx.is_null() {
                (lib.mpv_destroy)(mpv);
                return Err(format!("Fallo al crear mpv_render_context: {}", res));
            }

            // Load Video
            let load_cmd = format!("loadfile \"{}\" replace", video_path.replace('\\', "/"));
            let cmd_cstr = CString::new(load_cmd).unwrap();
            (lib.mpv_command_string)(mpv, cmd_cstr.as_ptr());

            let pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(width, height);
            let stride = (width * 4) as usize;

            Ok(Self {
                mpv,
                render_ctx,
                lib,
                _width: width,
                _height: height,
                size_param: [width as i32, height as i32],
                stride,
                format_cstr: CString::new("rgba").unwrap(),
                _api_type_cstr: api_type_cstr,
                pixel_buffer,
                is_playing: Arc::new(AtomicBool::new(true)),
            })
        }
    }

    pub fn render_frame(&mut self) -> Result<Image, String> {
        if !self.is_playing.load(Ordering::SeqCst) {
            return Ok(Image::from_rgba8(self.pixel_buffer.clone()));
        }

        unsafe {
            let raw_bytes = self.pixel_buffer.make_mut_bytes();
            let mut stride_val = self.stride;

            let mut render_params = [
                MpvRenderParam {
                    param_type: MPV_RENDER_PARAM_SW_SIZE,
                    data: self.size_param.as_mut_ptr() as *mut c_void,
                },
                MpvRenderParam {
                    param_type: MPV_RENDER_PARAM_SW_FORMAT,
                    data: self.format_cstr.as_ptr() as *mut c_void,
                },
                MpvRenderParam {
                    param_type: MPV_RENDER_PARAM_SW_STRIDE,
                    data: &mut stride_val as *mut usize as *mut c_void,
                },
                MpvRenderParam {
                    param_type: MPV_RENDER_PARAM_SW_POINTER,
                    data: raw_bytes.as_mut_ptr() as *mut c_void,
                },
                MpvRenderParam {
                    param_type: MPV_RENDER_PARAM_INVALID,
                    data: std::ptr::null_mut(),
                },
            ];

            let res = (self.lib.mpv_render_context_render)(self.render_ctx, render_params.as_mut_ptr());
            if res < 0 {
                return Err(format!("Error rendering mpv frame: {}", res));
            }

            Ok(Image::from_rgba8(self.pixel_buffer.clone()))
        }
    }

    pub fn render_frame_buffer(&mut self) -> Result<SharedPixelBuffer<Rgba8Pixel>, String> {
        if !self.is_playing.load(Ordering::SeqCst) {
            return Ok(self.pixel_buffer.clone());
        }

        unsafe {
            let raw_bytes = self.pixel_buffer.make_mut_bytes();
            let mut stride_val = self.stride;

            let mut render_params = [
                MpvRenderParam {
                    param_type: MPV_RENDER_PARAM_SW_SIZE,
                    data: self.size_param.as_mut_ptr() as *mut c_void,
                },
                MpvRenderParam {
                    param_type: MPV_RENDER_PARAM_SW_FORMAT,
                    data: self.format_cstr.as_ptr() as *mut c_void,
                },
                MpvRenderParam {
                    param_type: MPV_RENDER_PARAM_SW_STRIDE,
                    data: &mut stride_val as *mut usize as *mut c_void,
                },
                MpvRenderParam {
                    param_type: MPV_RENDER_PARAM_SW_POINTER,
                    data: raw_bytes.as_mut_ptr() as *mut c_void,
                },
                MpvRenderParam {
                    param_type: MPV_RENDER_PARAM_INVALID,
                    data: std::ptr::null_mut(),
                },
            ];

            let res = (self.lib.mpv_render_context_render)(self.render_ctx, render_params.as_mut_ptr());
            if res < 0 {
                return Err(format!("Error rendering mpv frame: {}", res));
            }

            Ok(self.pixel_buffer.clone())
        }
    }

    pub fn change_video(&mut self, video_path: &str) {
        let load_cmd = format!("loadfile \"{}\" replace", video_path.replace('\\', "/"));
        if let Ok(cmd_cstr) = CString::new(load_cmd) {
            unsafe {
                (self.lib.mpv_command_string)(self.mpv, cmd_cstr.as_ptr());
            }
        }
    }

    pub fn pause(&self) {
        let cmd = CString::new("set pause yes").unwrap();
        unsafe {
            (self.lib.mpv_command_string)(self.mpv, cmd.as_ptr());
        }
        self.is_playing.store(false, Ordering::SeqCst);
    }

    pub fn resume(&self) {
        let cmd = CString::new("set pause no").unwrap();
        unsafe {
            (self.lib.mpv_command_string)(self.mpv, cmd.as_ptr());
        }
        self.is_playing.store(true, Ordering::SeqCst);
    }
}

impl Drop for SlintMpvPlayer {
    fn drop(&mut self) {
        unsafe {
            if !self.render_ctx.is_null() {
                (self.lib.mpv_render_context_free)(self.render_ctx);
            }
            if !self.mpv.is_null() {
                (self.lib.mpv_destroy)(self.mpv);
            }
        }
    }
}

pub fn resolve_showroom_video_path() -> PathBuf {
    let candidate_paths = [
        PathBuf::from("public/assets/videos/landing.mp4"),
        PathBuf::from("dist_app/assets/videos/landing.mp4"),
        PathBuf::from("src/resources/videos/assets/generic/prom.mp4"),
        PathBuf::from("assets/videos/landing.mp4"),
    ];

    for path in &candidate_paths {
        if path.exists() {
            return path.clone();
        }
    }

    PathBuf::from("public/assets/videos/landing.mp4")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_showroom_video_path_not_empty() {
        let p = resolve_showroom_video_path();
        assert!(!p.to_string_lossy().is_empty());
    }

    #[test]
    fn test_mpv_library_load_fails_gracefully_without_panic() {
        // When running on a machine without mpv-2.dll, load() returns Err
        let res = MpvLibrary::load();
        // Should not panic, returns either Ok (if dll is present) or Err (if missing)
        match res {
            Ok(_) => println!("mpv library found on host!"),
            Err(e) => assert!(e.contains("libmpv no encontrado") || e.contains("Fallo")),
        }
    }
}
