use execution_engine::execute_system_status;
use policy_engine::PolicyContext;

#[tauri::command]
fn system_status() -> Result<system_status::SystemStatus, String> {
    let context = PolicyContext {
        ai_requested: false,
        user_confirmed: false,
    };

    execute_system_status(&context)
        .map(|result| result.status)
        .map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![system_status])
        .run(tauri::generate_context!())
        .expect("error while running Linux Powerhouse");
}
