use windows::core::HSTRING;
use windows::Data::Xml::Dom::XmlDocument;
use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};

use crate::assets;
use crate::notification::NotificationType;

const AUMID: &str = r"{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\WindowsPowerShell\v1.0\powershell.exe";

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Build Toast XML string with optional hero image and audio configuration.
///
/// This function is separated from `show_typed_toast` to allow unit testing
/// without requiring WinRT runtime.
fn build_toast_xml(
    title: &str,
    body: &str,
    attribution: &str,
    hero_path: Option<&str>,
    audio_src: &str,
    audio_loop: bool,
) -> String {
    let hero_image_element = match hero_path {
        Some(path) => format!(
            r#"<image placement="hero" src="file:///{}"/>"#,
            path
        ),
        None => String::new(),
    };

    let loop_attr = if audio_loop {
        r#" loop="true""#
    } else {
        r#" loop="false""#
    };

    format!(
        r#"<toast>
  <visual>
    <binding template="ToastGeneric">
      <text>{title}</text>
      <text>{body}</text>
      <text placement="attribution">{attribution}</text>
      {hero_image_element}
    </binding>
  </visual>
  <audio src="{audio_src}"{loop_attr}/>
</toast>"#,
        title = escape_xml(title),
        body = escape_xml(body),
        attribution = escape_xml(attribution),
        hero_image_element = hero_image_element,
        audio_src = audio_src,
        loop_attr = loop_attr,
    )
}

/// Display a typed Toast notification with hero image and per-type audio.
///
/// Uses `ensure_hero_image` to extract embedded assets on first run.
/// If the hero image cannot be written to disk, the toast displays without it (graceful degradation).
pub fn show_typed_toast(
    ntype: NotificationType,
    body: &str,
    attribution: &str,
) -> Result<(), crate::error::AppError> {
    let hero_path = assets::ensure_hero_image(ntype);
    let hero_path_str = hero_path.as_ref().map(|p| {
        p.to_string_lossy().replace('\\', "/")
    });

    let xml_string = build_toast_xml(
        ntype.title(),
        body,
        attribution,
        hero_path_str.as_deref(),
        ntype.audio_src(),
        ntype.audio_loop(),
    );

    let toast_xml = XmlDocument::new()?;
    toast_xml.LoadXml(&HSTRING::from(&*xml_string))?;
    let toast = ToastNotification::CreateToastNotification(&toast_xml)?;
    let notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(AUMID))?;
    notifier.Show(&toast)?;
    Ok(())
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

    #[test]
    fn xml_contains_hero_image() {
        let xml = build_toast_xml(
            "Task Complete",
            "Done",
            "my-project",
            Some("C:/Users/test/AppData/Local/claude-win-notify/assets/hero-task-complete.png"),
            "ms-winsoundevent:Notification.Default",
            false,
        );
        assert!(xml.contains(r#"<image placement="hero" src="file:///C:/Users/test/AppData/Local/claude-win-notify/assets/hero-task-complete.png"/>"#));
    }

    #[test]
    fn xml_omits_hero_when_none() {
        let xml = build_toast_xml(
            "Task Complete",
            "Done",
            "my-project",
            None,
            "ms-winsoundevent:Notification.Default",
            false,
        );
        assert!(!xml.contains("<image"));
    }

    #[test]
    fn xml_audio_default() {
        let xml = build_toast_xml(
            "Task Complete",
            "Done",
            "my-project",
            None,
            "ms-winsoundevent:Notification.Default",
            false,
        );
        assert!(xml.contains(r#"<audio src="ms-winsoundevent:Notification.Default""#));
    }

    #[test]
    fn xml_audio_error_no_loop() {
        let xml = build_toast_xml(
            "Error",
            "API rate limit",
            "my-project",
            None,
            "ms-winsoundevent:Notification.Looping.Alarm",
            false,
        );
        assert!(xml.contains(r#"<audio src="ms-winsoundevent:Notification.Looping.Alarm" loop="false"/>"#));
    }

    #[test]
    fn xml_escapes_special_chars_in_body() {
        let xml = build_toast_xml(
            "Title",
            "a<b>c&d",
            "project",
            None,
            "ms-winsoundevent:Notification.Default",
            false,
        );
        assert!(xml.contains("a&lt;b&gt;c&amp;d"));
        assert!(!xml.contains("a<b>c&d"));
    }

    #[test]
    fn xml_hero_path_uses_forward_slashes() {
        // Simulate what show_typed_toast does: replace backslashes with forward slashes
        let windows_path = r"C:\Users\test\AppData\Local\claude-win-notify\assets\hero-error.png";
        let forward_path = windows_path.replace('\\', "/");
        let xml = build_toast_xml(
            "Error",
            "Oops",
            "project",
            Some(&forward_path),
            "ms-winsoundevent:Notification.Looping.Alarm",
            false,
        );
        assert!(xml.contains("file:///C:/Users/test/AppData/Local/claude-win-notify/assets/hero-error.png"));
        assert!(!xml.contains('\\'));
    }

    #[test]
    fn xml_attribution_shows_project_name() {
        let xml = build_toast_xml(
            "Title",
            "Body",
            "claude-win-notify",
            None,
            "ms-winsoundevent:Notification.Default",
            false,
        );
        assert!(xml.contains(r#"<text placement="attribution">claude-win-notify</text>"#));
    }
}
