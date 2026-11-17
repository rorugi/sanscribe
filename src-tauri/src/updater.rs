//! Optional updates via tauri-plugin-updater. Without internet the check
//! simply fails quietly; the app keeps working offline.

use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_updater::UpdaterExt;

/// `interactive`: the user asked explicitly, so also report "up to date" and errors.
pub fn check(app: AppHandle, interactive: bool) {
    tauri::async_runtime::spawn(async move {
        let result = async {
            let updater = app.updater().map_err(|e| e.to_string())?;
            updater.check().await.map_err(|e| e.to_string())
        }
        .await;

        match result {
            Ok(Some(update)) => {
                let question = format!(
                    "Sanscribe {} is available (installed: {}).\n\nInstall it now? Sanscribe will restart.",
                    update.version, update.current_version
                );
                let app2 = app.clone();
                app.dialog()
                    .message(question)
                    .title("Update available")
                    .buttons(MessageDialogButtons::OkCancelCustom("Install".into(), "Later".into()))
                    .show(move |install| {
                        if !install {
                            return;
                        }
                        tauri::async_runtime::spawn(async move {
                            match update.download_and_install(|_, _| {}, || {}).await {
                                Ok(()) => app2.restart(),
                                Err(e) => {
                                    app2.dialog()
                                        .message(format!("The update failed:\n{e}"))
                                        .kind(MessageDialogKind::Error)
                                        .title("Sanscribe")
                                        .show(|_| {});
                                }
                            }
                        });
                    });
            }
            Ok(None) => {
                if interactive {
                    app.dialog().message("Sanscribe is up to date.").title("Sanscribe").show(|_| {});
                }
            }
            Err(e) => {
                log::info!("update check failed: {e}");
                if interactive {
                    app.dialog()
                        .message(format!("Could not check for updates:\n{e}"))
                        .kind(MessageDialogKind::Warning)
                        .title("Sanscribe")
                        .show(|_| {});
                }
            }
        }
    });
}
