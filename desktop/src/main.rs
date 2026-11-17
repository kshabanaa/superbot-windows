#![windows_subsystem = "windows"]
use std::{io::{Read, Write}, net::{TcpListener, TcpStream}, os::windows::process::CommandExt, path::PathBuf, process::{Child, Command, Stdio}, time::{Duration, Instant}};
use tao::{dpi::LogicalSize, event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder};
use wry::WebViewBuilder;

fn message(s: &str) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
    let body: Vec<u16> = s.encode_utf16().chain(Some(0)).collect();
    let title: Vec<u16> = "SuperBot".encode_utf16().chain(Some(0)).collect();
    unsafe { MessageBoxW(std::ptr::null_mut(), body.as_ptr(), title.as_ptr(), MB_OK | MB_ICONERROR); }
}
const BOTD: &[u8] = include_bytes!("../../target/release/botd.exe");
const BOTCTL: &[u8] = include_bytes!("../../target/release/botctl.exe");

fn install_backend(dir: &std::path::Path) -> Result<PathBuf, String> {
    let bin = dir.join("bin");
    std::fs::create_dir_all(&bin).map_err(|e| e.to_string())?;
    for (name, bytes) in [("botd.exe", BOTD), ("botctl.exe", BOTCTL)] {
        let path = bin.join(name);
        if std::fs::read(&path).ok().as_deref() != Some(bytes) {
            std::fs::write(&path, bytes).map_err(|e| format!("Could not prepare {name}: {e}"))?;
        }
    }
    Ok(bin.join("botd.exe"))
}

fn start() -> Result<(Child, String), String> {
    let dir = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).ok_or("LOCALAPPDATA is unavailable")?.join("SuperBot");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let exe = install_backend(&dir)?;
    let port = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?.local_addr().map_err(|e| e.to_string())?.port();
    let mut child = Command::new(exe).current_dir(&dir).env("PORT", port.to_string()).env("HOST", "127.0.0.1").env("SCREENS", "off").env("SUPERBOT_DESKTOP", "1")
        .env("DATA_DIR", dir.join("data")).env("WORKSPACE_DIR", dir.join("workspace"))
        .stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null()).creation_flags(0x08000000).spawn().map_err(|e| format!("Could not start SuperBot: {e}"))?;
    let addr = format!("127.0.0.1:{port}");
    let deadline = Instant::now() + Duration::from_secs(15);
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? { return Err(format!("SuperBot server stopped unexpectedly: {status}")); }
        if let Ok(mut stream) = TcpStream::connect(&addr) {
            let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
            let _ = stream.write_all(b"GET /healthz HTTP/1.0\r\nHost: localhost\r\n\r\n");
            let mut response = String::new();
            let _ = stream.read_to_string(&mut response);
            if response.contains("200 OK") && response.contains("\"ok\":true") { return Ok((child, format!("http://{addr}/"))); }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = child.kill();
    Err("SuperBot could not start. Please try again.".into())
}
fn main() {
    let name: Vec<u16> = "Local\\SuperBotDesktopWindow".encode_utf16().chain(Some(0)).collect();
    let mutex = unsafe { windows_sys::Win32::System::Threading::CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
    if mutex.is_null() { message("Could not start SuperBot."); return; }
    if unsafe { windows_sys::Win32::Foundation::GetLastError() } == 183 { message("SuperBot is already open."); return; }
    let (mut server, url) = match start() { Ok(v) => v, Err(e) => { message(&e); return; } };
    let event_loop = EventLoop::new();
    let window = match WindowBuilder::new().with_title("SuperBot").with_inner_size(LogicalSize::new(1200.0, 800.0)).with_min_inner_size(LogicalSize::new(700.0, 500.0)).build(&event_loop) {
        Ok(w) => w, Err(e) => { message(&format!("Could not open the window: {e}")); return; }
    };
    let webview = match WebViewBuilder::new().with_url(&url).build(&window) {
        Ok(v) => v, Err(e) => { message(&format!("Could not open the app. Microsoft Edge WebView2 Runtime is required.\n{e}")); return; }
    };
    event_loop.run(move |event, _, flow| {
        *flow = ControlFlow::Wait;
        let _ = (&window, &webview);
        if let Event::WindowEvent { event: WindowEvent::CloseRequested, .. } = event {
            drop(server.stdin.take());
            let deadline = Instant::now() + Duration::from_secs(8);
            while Instant::now() < deadline {
                if server.try_wait().ok().flatten().is_some() { break; }
                std::thread::sleep(Duration::from_millis(100));
            }
            let _ = server.kill();
            let _ = server.wait();
            *flow = ControlFlow::Exit;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extracts_both_backend_programs_and_repairs_changes() {
        let dir = std::env::temp_dir().join(format!("superbot-test-{}", std::process::id()));
        let exe = install_backend(&dir).unwrap();
        assert_eq!(std::fs::read(&exe).unwrap(), BOTD);
        assert_eq!(std::fs::read(dir.join("bin/botctl.exe")).unwrap(), BOTCTL);
        std::fs::write(&exe, b"broken").unwrap();
        install_backend(&dir).unwrap();
        assert_eq!(std::fs::read(&exe).unwrap(), BOTD);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
