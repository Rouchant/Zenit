use crate::MainWindow;
use slint::ComponentHandle;
use std::sync::Mutex;

static CUSTOM_COLORS: Mutex<[u32; 16]> = Mutex::new([0x00ffffff; 16]);

#[cfg(target_os = "windows")]
pub fn pick_color_dialog(initial_hex: &str) -> Option<String> {
    #[repr(C)]
    struct CHOOSECOLORW {
        l_struct_size: u32,
        hwnd_owner: *mut std::ffi::c_void,
        h_instance: *mut std::ffi::c_void,
        rgb_result: u32,
        lp_cust_colors: *mut u32,
        flags: u32,
        l_cust_data: isize,
        lpfn_hook: Option<unsafe extern "system" fn(*mut std::ffi::c_void, u32, usize, isize) -> usize>,
        lp_template_name: *const u16,
    }

    let clean = initial_hex.trim().trim_start_matches('#');
    let mut initial_rgb = 0x00ffffffu32;
    if clean.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&clean[0..2], 16),
            u8::from_str_radix(&clean[2..4], 16),
            u8::from_str_radix(&clean[4..6], 16),
        ) {
            // COLORREF is 0x00bbggrr
            initial_rgb = (r as u32) | ((g as u32) << 8) | ((b as u32) << 16);
        }
    }

    let mut custom = CUSTOM_COLORS.lock().unwrap();

    let mut cc = CHOOSECOLORW {
        l_struct_size: std::mem::size_of::<CHOOSECOLORW>() as u32,
        hwnd_owner: std::ptr::null_mut(),
        h_instance: std::ptr::null_mut(),
        rgb_result: initial_rgb,
        lp_cust_colors: custom.as_mut_ptr(),
        flags: 0x00000001 | 0x00000002, // CC_RGBINIT | CC_FULLOPEN
        l_cust_data: 0,
        lpfn_hook: None,
        lp_template_name: std::ptr::null(),
    };

    unsafe {
        if let Ok(lib) = libloading::Library::new("comdlg32.dll") {
            if let Ok(choose_color) = lib.get::<unsafe extern "system" fn(*mut CHOOSECOLORW) -> i32>(b"ChooseColorW\0") {
                let res = choose_color(&mut cc);
                if res != 0 {
                    let r = (cc.rgb_result & 0xff) as u8;
                    let g = ((cc.rgb_result >> 8) & 0xff) as u8;
                    let b = ((cc.rgb_result >> 16) & 0xff) as u8;
                    return Some(format!("#{:02x}{:02x}{:02x}", r, g, b));
                }
            }
        }
    }

    None
}

#[cfg(not(target_os = "windows"))]
pub fn pick_color_dialog(_initial_hex: &str) -> Option<String> {
    None
}

pub fn setup_color_picker_handlers(main_window: &MainWindow) {
    main_window.on_hex_to_color(|hex| {
        crate::app::sync::parse_hex_color(&hex, slint::Color::from_rgb_u8(100, 116, 139))
    });

    let handle = main_window.as_weak();
    main_window.on_pick_color_clicked(move |field| {
        if let Some(ui) = handle.upgrade() {
            let field_str = field.as_str();
            let current = match field_str {
                "bg_base" => ui.get_cfg_bg_base().to_string(),
                "bg_start" => ui.get_cfg_bg_start().to_string(),
                "bg_end" => ui.get_cfg_bg_end().to_string(),
                "card_bg" => ui.get_cfg_card_bg().to_string(),
                "card_border" => ui.get_cfg_card_border().to_string(),
                "text_primary" => ui.get_cfg_text_primary().to_string(),
                "text_secondary" => ui.get_cfg_text_secondary().to_string(),
                "accent" => ui.get_cfg_accent().to_string(),
                "btn_bg" => ui.get_cfg_btn_bg().to_string(),
                "btn_text" => ui.get_cfg_btn_text().to_string(),
                _ => String::new(),
            };

            if let Some(picked) = pick_color_dialog(&current) {
                match field_str {
                    "bg_base" => {
                        let (c1, c2, c3) = crate::app::sync::generate_analogous_gradient(&picked);
                        let s_c1 = crate::app::sync::parse_hex_color(&c1, slint::Color::from_rgb_u8(220, 237, 245));
                        let s_c2 = crate::app::sync::parse_hex_color(&c2, slint::Color::from_rgb_u8(227, 232, 245));
                        let s_c3 = crate::app::sync::parse_hex_color(&c3, slint::Color::from_rgb_u8(234, 228, 245));
                        ui.set_cfg_bg_base(picked.into());
                        ui.set_cfg_bg_start(c1.into());
                        ui.set_cfg_bg_mid(c2.into());
                        ui.set_cfg_bg_end(c3.into());
                        ui.set_theme_bg_start(s_c1);
                        ui.set_theme_bg_mid(s_c2);
                        ui.set_theme_bg_end(s_c3);
                    }
                    "bg_start" => ui.set_cfg_bg_start(picked.into()),
                    "bg_end" => ui.set_cfg_bg_end(picked.into()),
                    "card_bg" => ui.set_cfg_card_bg(picked.into()),
                    "card_border" => ui.set_cfg_card_border(picked.into()),
                    "text_primary" => ui.set_cfg_text_primary(picked.into()),
                    "text_secondary" => ui.set_cfg_text_secondary(picked.into()),
                    "accent" => ui.set_cfg_accent(picked.into()),
                    "btn_bg" => ui.set_cfg_btn_bg(picked.into()),
                    "btn_text" => ui.set_cfg_btn_text(picked.into()),
                    _ => {},
                }
            }
        }
    });

    let update_bg_handle = main_window.as_weak();
    main_window.on_update_bg_base_color(move |base_hex| {
        if let Some(ui) = update_bg_handle.upgrade() {
            let (c1, c2, c3) = crate::app::sync::generate_analogous_gradient(&base_hex);
            let s_c1 = crate::app::sync::parse_hex_color(&c1, slint::Color::from_rgb_u8(220, 237, 245));
            let s_c2 = crate::app::sync::parse_hex_color(&c2, slint::Color::from_rgb_u8(227, 232, 245));
            let s_c3 = crate::app::sync::parse_hex_color(&c3, slint::Color::from_rgb_u8(234, 228, 245));
            ui.set_cfg_bg_base(base_hex);
            ui.set_cfg_bg_start(c1.into());
            ui.set_cfg_bg_mid(c2.into());
            ui.set_cfg_bg_end(c3.into());
            ui.set_theme_bg_start(s_c1);
            ui.set_theme_bg_mid(s_c2);
            ui.set_theme_bg_end(s_c3);
        }
    });
}
