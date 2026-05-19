use windows::core::*;
use windows::Data::Xml::Dom::XmlDocument;
use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};

fn main() -> Result<()> {
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
