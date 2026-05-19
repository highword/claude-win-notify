use std::env;
use std::thread;
use std::time::Duration;
use url::Url;
use windows::core::*;
use windows::Data::Xml::Dom::XmlDocument;
use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};
use windows::Win32::Foundation::*;
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // Mode 2: Protocol activation — parse URI from --focus argument
    if args.len() >= 3 && args[1] == "--focus" {
        return handle_focus(&args[2]);
    }

    // Mode 1: Show toast notification (default behavior)
    show_toast()
}

fn handle_focus(uri_str: &str) -> Result<()> {
    let url = Url::parse(uri_str).expect("Failed to parse URI");

    let session = url
        .query_pairs()
        .find(|(k, _)| k == "session")
        .map(|(_, v)| v.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let pid_str = url
        .query_pairs()
        .find(|(k, _)| k == "pid")
        .map(|(_, v)| v.to_string())
        .unwrap_or_else(|| "0".to_string());

    println!("PROTOCOL ACTIVATED:");
    println!("  Full URI: {}", uri_str);
    println!("  Session: {}", session);
    println!("  PID: {}", pid_str);

    // Print any additional parameters (e.g., cwd with CJK characters)
    for (key, value) in url.query_pairs() {
        if key != "session" && key != "pid" {
            println!("  {}: {}", key, value);
        }
    }

    // --- SetForegroundWindow with fallback chain ---
    let target_pid: u32 = pid_str.parse().unwrap_or(0);
    if target_pid > 0 {
        println!("\nFOCUS: Attempting to bring PID {} to foreground...", target_pid);

        let hwnd = find_window_by_pid(target_pid);
        match hwnd {
            None => {
                println!("  ERROR: No visible window found for PID {}", target_pid);
                std::process::exit(1);
            }
            Some(hwnd) => {
                println!("  Found window handle: 0x{:X}", hwnd.0 as usize);
                let success = focus_window(hwnd);
                thread::sleep(Duration::from_millis(100));

                let fg_after = unsafe { GetForegroundWindow() };
                let verified = fg_after == hwnd;
                println!("  Foreground after focus: 0x{:X}", fg_after.0 as usize);
                if verified {
                    println!("  RESULT: PASS (foreground verified)");
                } else if success {
                    println!("  RESULT: PARTIAL (strategy reported success but foreground mismatch)");
                    std::process::exit(2);
                } else {
                    println!("  RESULT: FLASH (taskbar flash only)");
                    std::process::exit(2);
                }
            }
        }
    } else {
        println!("SUCCESS: Protocol activation parsed correctly (Rust)");
    }

    Ok(())
}

fn find_window_by_pid(target_pid: u32) -> Option<HWND> {
    // Use raw pointer via LPARAM to pass data to the EnumWindows callback
    struct EnumData {
        target_pid: u32,
        found: HWND,
    }

    let mut data = EnumData {
        target_pid,
        found: HWND::default(),
    };

    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let data = &mut *(lparam.0 as *mut EnumData);
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == data.target_pid && IsWindowVisible(hwnd).as_bool() {
            data.found = hwnd;
            return BOOL::from(false); // stop enumeration
        }
        BOOL::from(true) // continue
    }

    unsafe {
        let _ = EnumWindows(
            Some(enum_proc),
            LPARAM(&mut data as *mut EnumData as isize),
        );
    }

    if data.found.0 == std::ptr::null_mut() {
        None
    } else {
        Some(data.found)
    }
}

fn focus_window(hwnd: HWND) -> bool {
    unsafe {
        // Restore if minimized
        if IsIconic(hwnd).as_bool() {
            println!("  Window is minimized, restoring...");
            let _ = ShowWindow(hwnd, SW_RESTORE);
            thread::sleep(Duration::from_millis(50));
        }

        // Strategy 1: Direct SetForegroundWindow
        let _ = SetForegroundWindow(hwnd);
        thread::sleep(Duration::from_millis(50));
        if GetForegroundWindow() == hwnd {
            println!("  Strategy 1 (direct): SUCCESS");
            return true;
        }

        // Strategy 2: AttachThreadInput
        let fg_hwnd = GetForegroundWindow();
        let fg_tid = GetWindowThreadProcessId(fg_hwnd, None);
        let our_tid = GetCurrentThreadId();
        let _ = AttachThreadInput(our_tid, fg_tid, true);
        let _ = SetForegroundWindow(hwnd);
        let _ = AttachThreadInput(our_tid, fg_tid, false);
        thread::sleep(Duration::from_millis(50));
        if GetForegroundWindow() == hwnd {
            println!("  Strategy 2 (AttachThreadInput): SUCCESS");
            return true;
        }

        // Strategy 3: Alt key hack
        keybd_event(
            VK_MENU.0 as u8,
            0,
            KEYBD_EVENT_FLAGS(KEYEVENTF_EXTENDEDKEY.0),
            0,
        );
        keybd_event(
            VK_MENU.0 as u8,
            0,
            KEYBD_EVENT_FLAGS(KEYEVENTF_KEYUP.0),
            0,
        );
        let _ = SetForegroundWindow(hwnd);
        thread::sleep(Duration::from_millis(50));
        if GetForegroundWindow() == hwnd {
            println!("  Strategy 3 (Alt key hack): SUCCESS");
            return true;
        }

        // Strategy 4: Minimize then restore
        let _ = ShowWindow(hwnd, SW_MINIMIZE);
        thread::sleep(Duration::from_millis(50));
        let _ = ShowWindow(hwnd, SW_RESTORE);
        thread::sleep(Duration::from_millis(50));
        if GetForegroundWindow() == hwnd {
            println!("  Strategy 4 (minimize/restore): SUCCESS");
            return true;
        }

        println!("  All strategies attempted, window may only flash in taskbar");
        false
    }
}

fn show_toast() -> Result<()> {
    // Use PowerShell's AUMID for spike (avoids Start Menu shortcut requirement)
    let aumid = "{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe";

    let toast_xml = XmlDocument::new()?;
    toast_xml.LoadXml(&HSTRING::from(
        r#"<toast>
          <visual>
            <binding template="ToastGeneric">
              <text>Claude Code [Rust Spike]</text>
              <text>Toast notification via windows-rs — integration point #1 validated!</text>
            </binding>
          </visual>
          <audio src="ms-winsoundevent:Notification.Default"/>
        </toast>"#,
    ))?;

    let toast = ToastNotification::CreateToastNotification(&toast_xml)?;
    let notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(aumid))?;
    notifier.Show(&toast)?;

    println!("SUCCESS: Toast notification displayed (Rust)");
    Ok(())
}
