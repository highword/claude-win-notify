using System;
using System.Linq;
using Windows.UI.Notifications;
using Windows.Data.Xml.Dom;

if (args.Length > 0 && args[0] == "--version")
{
    Console.WriteLine("CSharpSpike v0.1");
    return;
}

// Mode 2: Protocol activation — parse URI from --focus argument
if (args.Length >= 2 && args[0] == "--focus")
{
    var uri = new Uri(args[1]);
    var queryString = uri.Query.TrimStart('?');
    var pairs = queryString.Split('&')
        .Where(p => !string.IsNullOrEmpty(p))
        .Select(p => p.Split('=', 2))
        .ToDictionary(p => p[0], p => Uri.UnescapeDataString(p.Length > 1 ? p[1] : ""));

    var session = pairs.ContainsKey("session") ? pairs["session"] : "unknown";
    var pid = pairs.ContainsKey("pid") ? pairs["pid"] : "0";

    Console.WriteLine("PROTOCOL ACTIVATED:");
    Console.WriteLine($"  Full URI: {args[1]}");
    Console.WriteLine($"  Session: {session}");
    Console.WriteLine($"  PID: {pid}");

    // Print any additional parameters (e.g., cwd with CJK characters)
    foreach (var pair in pairs.Where(p => p.Key != "session" && p.Key != "pid"))
    {
        Console.WriteLine($"  {pair.Key}: {pair.Value}");
    }

    Console.WriteLine("SUCCESS: Protocol activation parsed correctly (C#)");
    return;
}

// Mode 1: Show toast notification (default behavior)
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
