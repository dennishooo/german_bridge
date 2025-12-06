use std::net::IpAddr;
use pnet::datalink;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn get_local_ip() -> Option<String> {
    for iface in datalink::interfaces() {
        for ip in iface.ips {
            match ip.ip() {
                IpAddr::V4(addr) => {
                    if !addr.is_loopback() && !addr.is_link_local() {
                        return Some(addr.to_string());
                    }
                }
                _ => {}
            }
        }
    }
    None
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Some(ip) = get_local_ip() {
        println!("🌐 App available at: http://{}:1420", ip);
    }
    println!("🌐 Local access: http://localhost:1420");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
