using System;
using Windows.UI.Notifications;
using Windows.Data.Xml.Dom;

if (args.Length > 0 && args[0] == "--version")
{
    Console.WriteLine("CSharpSpike v0.1");
    return;
}

// Use PowerShell's AUMID for spike (avoids Start Menu shortcut requirement)
const string AUMID = "{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe";

var toastXml = new XmlDocument();
toastXml.LoadXml(@"
<toast>
  <visual>
    <binding template='ToastGeneric'>
      <text>Claude Code [C# Spike]</text>
      <text>Toast notification via NativeAOT — integration point #1 validated!</text>
    </binding>
  </visual>
  <audio src='ms-winsoundevent:Notification.Default'/>
</toast>");

var toast = new ToastNotification(toastXml);
var notifier = ToastNotificationManager.CreateToastNotifier(AUMID);
notifier.Show(toast);

Console.WriteLine("SUCCESS: Toast notification displayed (C# NativeAOT)");
