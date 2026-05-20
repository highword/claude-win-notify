use windows::core::HSTRING;
use windows::Data::Xml::Dom::XmlDocument;
use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};

const AUMID: &str = r"{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\WindowsPowerShell\v1.0\powershell.exe";

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn show_toast(title: &str, body: &str, attribution: &str) -> Result<(), crate::error::AppError> {
    let xml_string = format!(
        r#"<toast>
  <visual>
    <binding template="ToastGeneric">
      <text>{}</text>
      <text>{}</text>
      <text placement="attribution">{}</text>
    </binding>
  </visual>
  <audio src="ms-winsoundevent:Notification.Default"/>
</toast>"#,
        escape_xml(title),
        escape_xml(body),
        escape_xml(attribution)
    );

    let toast_xml = XmlDocument::new()?;
    toast_xml.LoadXml(&HSTRING::from(&*xml_string))?;
    let toast = ToastNotification::CreateToastNotification(&toast_xml)?;
    let notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(AUMID))?;
    notifier.Show(&toast)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_xml_special_chars() {
        assert_eq!(
            escape_xml("a<b>c&d\"e'f"),
            "a&lt;b&gt;c&amp;d&quot;e&apos;f"
        );
    }

    #[test]
    fn test_escape_xml_cjk_passthrough() {
        assert_eq!(escape_xml("D:\\项目\\测试"), "D:\\项目\\测试");
    }

    #[test]
    fn test_escape_xml_empty() {
        assert_eq!(escape_xml(""), "");
    }
}
