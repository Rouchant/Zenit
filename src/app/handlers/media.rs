use crate::app::sync::populate_vault_videos;
use crate::MainWindow;
use slint::ComponentHandle;

pub fn setup_media_handlers(main_window: &MainWindow) {
    let spawn_video_picker =
        |ui_weak: slint::Weak<MainWindow>,
         setter: Box<dyn Fn(&MainWindow, String) + Send + 'static>| {
            std::thread::spawn(move || {
                if let Some(picked) = crate::domains::media::vault::pick_video_file() {
                    let chosen_path =
                        if let Ok(item) = crate::domains::media::vault::import_video_to_vault(&picked) {
                            item.path
                        } else {
                            picked
                        };
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            setter(&ui, chosen_path);
                            populate_vault_videos(&ui);
                        }
                    });
                }
            });
        };

    let h_central = main_window.as_weak();
    let picker_central = spawn_video_picker.clone();
    main_window.on_pick_central_video_clicked(move || {
        picker_central(
            h_central.clone(),
            Box::new(|ui, path| {
                ui.set_cfg_central_video(path.into());
            }),
        );
    });

    let h_bg = main_window.as_weak();
    let picker_bg = spawn_video_picker.clone();
    main_window.on_pick_bg_video_clicked(move || {
        picker_bg(
            h_bg.clone(),
            Box::new(|ui, path| {
                ui.set_cfg_bg_video(path.into());
            }),
        );
    });

    let h_inact1 = main_window.as_weak();
    let picker_inact1 = spawn_video_picker.clone();
    main_window.on_pick_inactivity_1_clicked(move || {
        picker_inact1(
            h_inact1.clone(),
            Box::new(|ui, path| {
                ui.set_cfg_inactivity_video_1(path.into());
            }),
        );
    });

    let h_inact2 = main_window.as_weak();
    let picker_inact2 = spawn_video_picker.clone();
    main_window.on_pick_inactivity_2_clicked(move || {
        picker_inact2(
            h_inact2.clone(),
            Box::new(|ui, path| {
                ui.set_cfg_inactivity_video_2(path.into());
            }),
        );
    });

    let h_inact3 = main_window.as_weak();
    let picker_inact3 = spawn_video_picker.clone();
    main_window.on_pick_inactivity_3_clicked(move || {
        picker_inact3(
            h_inact3.clone(),
            Box::new(|ui, path| {
                ui.set_cfg_inactivity_video_3(path.into());
            }),
        );
    });

    // Vault: Import video to safe folder
    let vault_handle = main_window.as_weak();
    main_window.on_import_video_to_vault_clicked(move || {
        let wh = vault_handle.clone();
        std::thread::spawn(move || {
            if let Some(picked) = crate::domains::media::vault::pick_video_file() {
                if let Ok(_item) = crate::domains::media::vault::import_video_to_vault(&picked) {
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = wh.upgrade() {
                            populate_vault_videos(&ui);
                        }
                    });
                }
            }
        });
    });

    // Vault: Delete video
    let del_vault_handle = main_window.as_weak();
    main_window.on_delete_vault_video_clicked(move |filename| {
        let _ = crate::domains::media::vault::delete_vault_video(&filename);
        if let Some(ui) = del_vault_handle.upgrade() {
            populate_vault_videos(&ui);
        }
    });
}
