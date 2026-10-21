use libloading::{Library, Symbol};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
use std::ffi::{c_char, c_int, c_void, CString};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub const MPV_RENDER_PARAM_INVALID: u32 = 0;
pub const MPV_RENDER_PARAM_API_TYPE: u32 = 1;
pub const MPV_RENDER_PARAM_SW_SIZE: u32 = 17;
pub const MPV_RENDER_PARAM_SW_FORMAT: u32 = 18;
pub const MPV_RENDER_PARAM_SW_STRIDE: u32 = 19;
pub const MPV_RENDER_PARAM_SW_POINTER: u32 = 20;

pub const MPV_RENDER_API_TYPE_SW: &str = "sw";

#[repr(C)]
pub struct MpvRenderParam {
    pub param_type: u32,
    pub data: *mut c_void,
}

type MpvCreateFn = unsafe extern "C" fn() -> *mut c_void;
type MpvInitializeFn = unsafe extern "C" fn(*mut c_void) -> c_int;
type MpvDestroyFn = unsafe extern "C" fn(*mut c_void);
type MpvCommandFn = unsafe extern "C" fn(*mut c_void, *const *const c_char) -> c_int;
type MpvCommandStringFn = unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int;
type MpvSetOptionStringFn = unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> c_int;
type MpvRenderContextCreateFn = unsafe extern "C" fn(*mut *mut c_void, *mut c_void, *mut MpvRenderParam) -> c_int;
type MpvRenderContextRenderFn = unsafe extern "C" fn(*mut c_void, *mut MpvRenderParam) -> c_int;
type MpvRenderContextUpdateFn = unsafe extern "C" fn(*mut c_void) -> u64;
type MpvRenderContextFreeFn = unsafe extern "C" fn(*mut c_void);
type MpvGetPropertyStringFn = unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_char;
type MpvFreeFn = unsafe extern "C" fn(*mut c_void);

pub struct MpvLibrary {
    _lib: Library,
    pub mpv_create: MpvCreateFn,
    pub mpv_initialize: MpvInitializeFn,
    pub mpv_destroy: MpvDestroyFn,
    pub mpv_command: MpvCommandFn,
    pub mpv_command_string: MpvCommandStringFn,
    pub mpv_set_option_string: MpvSetOptionStringFn,
    pub mpv_render_context_create: MpvRenderContextCreateFn,
    pub mpv_render_context_render: MpvRenderContextRenderFn,
    pub mpv_render_context_update: Option<MpvRenderContextUpdateFn>,
    pub mpv_render_context_free: MpvRenderContextFreeFn,
    pub mpv_get_property_string: Option<MpvGetPropertyStringFn>,
    pub mpv_free: Option<MpvFreeFn>,
}

impl MpvLibrary {
    pub fn load() -> Result<Self, String> {
        let candidates = [
            "libmpv-2.dll",
            "mpv-2.dll",
            "libmpv-1.dll",
            "mpv-1.dll",
            "mpv.dll",
        ];

        // Search relative to exe, cwd, and standard locations
        let search_dirs = [
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            std::env::current_exe().map(|p| p.parent().unwrap_or(Path::new(".")).to_path_buf()).unwrap_or_else(|_| PathBuf::from(".")),
            PathBuf::from("."),
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
            "libmpv no encontrado. Coloca libmpv-2.dll o mpv-2.dll en la carpeta de la aplicación.".to_string()
        })?;

        unsafe {
            let mpv_create: Symbol<MpvCreateFn> = lib.get(b"mpv_create\0").map_err(|e| e.to_string())?;
            let mpv_initialize: Symbol<MpvInitializeFn> = lib.get(b"mpv_initialize\0").map_err(|e| e.to_string())?;
            let mpv_destroy: Symbol<MpvDestroyFn> = lib.get(b"mpv_destroy\0").map_err(|e| e.to_string())?;
            let mpv_command: Symbol<MpvCommandFn> = lib.get(b"mpv_command\0").map_err(|e| e.to_string())?;
            let mpv_command_string: Symbol<MpvCommandStringFn> = lib.get(b"mpv_command_string\0").map_err(|e| e.to_string())?;
            let mpv_set_option_string: Symbol<MpvSetOptionStringFn> = lib.get(b"mpv_set_option_string\0").map_err(|e| e.to_string())?;
            let mpv_render_context_create: Symbol<MpvRenderContextCreateFn> = lib.get(b"mpv_render_context_create\0").map_err(|e| e.to_string())?;
            let mpv_render_context_render: Symbol<MpvRenderContextRenderFn> = lib.get(b"mpv_render_context_render\0").map_err(|e| e.to_string())?;
            let mpv_render_context_update: Option<MpvRenderContextUpdateFn> = lib.get(b"mpv_render_context_update\0").ok().map(|s: Symbol<MpvRenderContextUpdateFn>| *s);
            let mpv_render_context_free: Symbol<MpvRenderContextFreeFn> = lib.get(b"mpv_render_context_free\0").map_err(|e| e.to_string())?;
            let mpv_get_property_string: Option<MpvGetPropertyStringFn> = lib.get(b"mpv_get_property_string\0").ok().map(|s: Symbol<MpvGetPropertyStringFn>| *s);
            let mpv_free: Option<MpvFreeFn> = lib.get(b"mpv_free\0").ok().map(|s: Symbol<MpvFreeFn>| *s);

            Ok(Self {
                mpv_create: *mpv_create,
                mpv_initialize: *mpv_initialize,
                mpv_destroy: *mpv_destroy,
                mpv_command: *mpv_command,
                mpv_command_string: *mpv_command_string,
                mpv_set_option_string: *mpv_set_option_string,
                mpv_render_context_create: *mpv_render_context_create,
                mpv_render_context_render: *mpv_render_context_render,
                mpv_render_context_update,
                mpv_render_context_free: *mpv_render_context_free,
                mpv_get_property_string,
                mpv_free,
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
    first_frame: bool,
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
            let opt_auto = CString::new("auto-safe").unwrap();
            (lib.mpv_set_option_string)(mpv, opt_hwdec.as_ptr(), opt_auto.as_ptr());

            let opt_threads_key = CString::new("vd-lavc-threads").unwrap();
            let opt_threads_val = CString::new("4").unwrap();
            (lib.mpv_set_option_string)(mpv, opt_threads_key.as_ptr(), opt_threads_val.as_ptr());

            let opt_framedrop_key = CString::new("framedrop").unwrap();
            let opt_framedrop_val = CString::new("vo").unwrap();
            (lib.mpv_set_option_string)(mpv, opt_framedrop_key.as_ptr(), opt_framedrop_val.as_ptr());

            let opt_vo = CString::new("vo").unwrap();
            let opt_libmpv = CString::new("libmpv").unwrap();
            (lib.mpv_set_option_string)(mpv, opt_vo.as_ptr(), opt_libmpv.as_ptr());

            let opt_no = CString::new("no").unwrap();

            // Disable all OSD (On-Screen Display) bars and text completely
            let opt_osc = CString::new("osc").unwrap();
            (lib.mpv_set_option_string)(mpv, opt_osc.as_ptr(), opt_no.as_ptr());

            let opt_osd_bar = CString::new("osd-bar").unwrap();
            (lib.mpv_set_option_string)(mpv, opt_osd_bar.as_ptr(), opt_no.as_ptr());

            let opt_osd_on_seek = CString::new("osd-on-seek").unwrap();
            (lib.mpv_set_option_string)(mpv, opt_osd_on_seek.as_ptr(), opt_no.as_ptr());

            let opt_osd_level = CString::new("osd-level").unwrap();
            let opt_zero = CString::new("0").unwrap();
            (lib.mpv_set_option_string)(mpv, opt_osd_level.as_ptr(), opt_zero.as_ptr());

            // Mute all video audio completely
            let opt_mute = CString::new("mute").unwrap();
            let opt_yes = CString::new("yes").unwrap();
            (lib.mpv_set_option_string)(mpv, opt_mute.as_ptr(), opt_yes.as_ptr());

            let opt_volume = CString::new("volume").unwrap();
            (lib.mpv_set_option_string)(mpv, opt_volume.as_ptr(), opt_zero.as_ptr());

            let opt_ao = CString::new("ao").unwrap();
            let opt_null = CString::new("null").unwrap();
            (lib.mpv_set_option_string)(mpv, opt_ao.as_ptr(), opt_null.as_ptr());

            let opt_aid = CString::new("aid").unwrap();
            (lib.mpv_set_option_string)(mpv, opt_aid.as_ptr(), opt_no.as_ptr());

            let opt_audio = CString::new("audio").unwrap();
            (lib.mpv_set_option_string)(mpv, opt_audio.as_ptr(), opt_no.as_ptr());

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

            // Load Video using mpv_command array for robust path handling
            let c_cmd = CString::new("loadfile").unwrap();
            let c_path = CString::new(video_path.replace('\\', "/")).unwrap();
            let c_mode = CString::new("replace").unwrap();
            let cmd_args: [*const c_char; 4] = [
                c_cmd.as_ptr(),
                c_path.as_ptr(),
                c_mode.as_ptr(),
                std::ptr::null(),
            ];
            (lib.mpv_command)(mpv, cmd_args.as_ptr());

            let mut pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(width, height);
            // Pre-fill alpha = 255 for opaque rendering
            for chunk in pixel_buffer.make_mut_bytes().chunks_exact_mut(4) {
                chunk[3] = 255;
            }

            let stride = (width * 4) as usize;

            Ok(Self {
                mpv,
                render_ctx,
                lib,
                _width: width,
                _height: height,
                size_param: [width as i32, height as i32],
                stride,
                format_cstr: CString::new("rgb0").unwrap(),
                _api_type_cstr: api_type_cstr,
                pixel_buffer,
                is_playing: Arc::new(AtomicBool::new(true)),
                first_frame: true,
            })
        }
    }

    pub fn render_frame(&mut self) -> Result<Image, String> {
        let buf = self.render_frame_buffer()?.unwrap_or_else(|| self.pixel_buffer.clone());
        Ok(Image::from_rgba8(buf))
    }

    pub fn render_frame_buffer(&mut self) -> Result<Option<SharedPixelBuffer<Rgba8Pixel>>, String> {
        if !self.is_playing.load(Ordering::SeqCst) {
            return Ok(None);
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

            // In rgb0: ensure byte 3 (alpha) is 255 (0xFF) so Slint renders 100% opaque video.
            // Using 32-bit integer chunks allows compiler auto-vectorization (SIMD).
            let (_, pixels, _) = raw_bytes.align_to_mut::<u32>();
            for pixel in pixels {
                *pixel |= 0xFF00_0000;
            }

            Ok(Some(self.pixel_buffer.clone()))
        }
    }

    pub fn change_video(&mut self, video_path: &str) {
        self.change_video_looping(video_path, true);
    }

    pub fn change_video_looping(&mut self, video_path: &str, looping: bool) {
        self.first_frame = true;
        let loop_cmd = CString::new(if looping {
            "set loop-file inf"
        } else {
            "set loop-file no"
        }).unwrap();
        let keep_cmd = CString::new(if looping {
            "set keep-open no"
        } else {
            "set keep-open yes"
        }).unwrap();
        let unpause_cmd = CString::new("set pause no").unwrap();
        let mute_cmd = CString::new("no-osd set mute yes").unwrap();
        let aid_cmd = CString::new("no-osd set aid no").unwrap();

        unsafe {
            (self.lib.mpv_command_string)(self.mpv, loop_cmd.as_ptr());
            (self.lib.mpv_command_string)(self.mpv, keep_cmd.as_ptr());
            (self.lib.mpv_command_string)(self.mpv, unpause_cmd.as_ptr());
            (self.lib.mpv_command_string)(self.mpv, mute_cmd.as_ptr());
            (self.lib.mpv_command_string)(self.mpv, aid_cmd.as_ptr());
        }
        self.is_playing.store(true, Ordering::SeqCst);

        if let (Ok(c_cmd), Ok(c_path), Ok(c_mode)) = (
            CString::new("loadfile"),
            CString::new(video_path.replace('\\', "/")),
            CString::new("replace"),
        ) {
            let cmd_args: [*const c_char; 4] = [
                c_cmd.as_ptr(),
                c_path.as_ptr(),
                c_mode.as_ptr(),
                std::ptr::null(),
            ];
            unsafe {
                (self.lib.mpv_command)(self.mpv, cmd_args.as_ptr());
            }
        }
    }

    pub fn get_property_string(&self, name: &str) -> Option<String> {
        let get_fn = self.lib.mpv_get_property_string?;
        let free_fn = self.lib.mpv_free?;
        if let Ok(c_name) = CString::new(name) {
            unsafe {
                let ptr = get_fn(self.mpv, c_name.as_ptr());
                if !ptr.is_null() {
                    let s = std::ffi::CStr::from_ptr(ptr).to_string_lossy().to_string();
                    free_fn(ptr as *mut c_void);
                    return Some(s);
                }
            }
        }
        None
    }

    pub fn is_eof(&self) -> bool {
        if let Some(val) = self.get_property_string("eof-reached") {
            if val == "yes" {
                return true;
            }
        }
        if let Some(val) = self.get_property_string("idle-active") {
            if val == "yes" {
                return true;
            }
        }
        false
    }

    pub fn get_duration(&self) -> Option<f64> {
        self.get_property_string("duration").and_then(|s| s.parse::<f64>().ok())
    }

    pub fn get_time_pos(&self) -> Option<f64> {
        self.get_property_string("time-pos").and_then(|s| s.parse::<f64>().ok())
    }

    pub fn seek(&self, sec: f64) {
        if let Ok(cmd) = CString::new(format!("seek {} absolute", sec)) {
            unsafe {
                (self.lib.mpv_command_string)(self.mpv, cmd.as_ptr());
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
    let cfg = crate::domains::showcase::config::load_config();
    if !cfg.central_video_path.trim().is_empty() {
        let p = resolve_video_path(&cfg.central_video_path);
        if p.exists() {
            return p;
        }
    }

    let candidate_paths = [
        PathBuf::from("assets/videos/landing2.mp4"),
        PathBuf::from("assets/videos/landing.mp4"),
        PathBuf::from("assets/videos/promo2.mp4"),
        PathBuf::from("assets/videos/promo.mp4"),
        PathBuf::from("assets/videos/bg.mp4"),
    ];

    let base = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    for path in &candidate_paths {
        let full = base.join(path);
        if full.exists() {
            return full;
        }
        if path.exists() {
            return path.clone();
        }
    }

    base.join("assets/videos/landing2.mp4")
}

pub fn resolve_video_path(configured_path: &str) -> PathBuf {
    let candidate = PathBuf::from(configured_path);
    let base = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let full = base.join(&candidate);
    if full.exists() {
        return full;
    }
    if candidate.exists() {
        return candidate;
    }
    base.join(configured_path)
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
        let res = MpvLibrary::load();
        match res {
            Ok(_) => println!("mpv library found on host!"),
            Err(e) => assert!(e.contains("libmpv no encontrado") || e.contains("Fallo")),
        }
    }
}
