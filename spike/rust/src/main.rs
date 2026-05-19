use std::env;
use url::Url;
use windows::core::*;
use windows::Data::Xml::Dom::XmlDocument;
use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};

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

    let pid = url
        .query_pairs()
        .find(|(k, _)| k == "pid")
        .map(|(_, v)| v.to_string())
        .unwrap_or_else(|| "0".to_string());

    println!("PROTOCOL ACTIVATED:");
    println!("  Full URI: {}", uri_str);
    println!("  Session: {}", session);
    println!("  PID: {}", pid);

    // Print any additional parameters (e.g., cwd with CJK characters)
    for (key, value) in url.query_pairs() {
        if key != "session" && key != "pid" {
            println!("  {}: {}", key, value);
        }
    }

    println!("SUCCESS: Protocol activation parsed correctly (Rust)");
    Ok(())
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
