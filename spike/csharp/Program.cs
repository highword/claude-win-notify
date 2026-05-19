using System;
using System.Linq;
using System.Runtime.InteropServices;
using Windows.UI.Notifications;
using Windows.Data.Xml.Dom;

if (args.Length > 0 && args[0] == "--version")
{
    Console.WriteLine("CSharpSpike v0.2");
    return;
}

// Mode 2: Protocol activation — parse URI from --focus argument, then focus window
if (args.Length >= 2 && args[0] == "--focus")
{
    var uri = new Uri(args[1]);
    var queryString = uri.Query.TrimStart('?');
    var pairs = queryString.Split('&')
        .Where(p => !string.IsNullOrEmpty(p))
        .Select(p => p.Split('=', 2))
        .ToDictionary(p => p[0], p => Uri.UnescapeDataString(p.Length > 1 ? p[1] : ""));

    var session = pairs.ContainsKey("session") ? pairs["session"] : "unknown";
    var pidStr = pairs.ContainsKey("pid") ? pairs["pid"] : "0";

    Console.WriteLine("PROTOCOL ACTIVATED:");
    Console.WriteLine($"  Full URI: {args[1]}");
    Console.WriteLine($"  Session: {session}");
    Console.WriteLine($"  PID: {pidStr}");

    // Print any additional parameters (e.g., cwd with CJK characters)
    foreach (var pair in pairs.Where(p => p.Key != "session" && p.Key != "pid"))
    {
        Console.WriteLine($"  {pair.Key}: {pair.Value}");
    }

    // --- SetForegroundWindow with fallback chain ---
    if (uint.TryParse(pidStr, out uint targetPid) && targetPid > 0)
    {
        Console.WriteLine($"\nFOCUS: Attempting to bring PID {targetPid} to foreground...");
        var hwnd = WindowFocus.FindMainWindowByPid(targetPid);
        if (hwnd == IntPtr.Zero)
        {
            Console.WriteLine("  ERROR: No visible window found for PID " + targetPid);
            Environment.ExitCode = 1;
            return;
        }
        Console.WriteLine($"  Found window handle: 0x{hwnd.ToInt64():X}");

        bool success = WindowFocus.FocusWindow(hwnd);
        // Verify by checking if target is now the foreground window
        System.Threading.Thread.Sleep(100);
        var fgAfter = WindowFocus.GetForegroundWindow();
        bool verified = fgAfter == hwnd;
        Console.WriteLine($"  Foreground after focus: 0x{fgAfter.ToInt64():X}");
        Console.WriteLine($"  RESULT: {(verified ? "PASS (foreground verified)" : success ? "PARTIAL (strategy reported success but foreground mismatch)" : "FLASH (taskbar flash only)")}");
        Environment.ExitCode = verified ? 0 : 2;
    }
    else
    {
        Console.WriteLine("SUCCESS: Protocol activation parsed correctly (C#)");
    }
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

// --- WindowFocus helper class with P/Invoke declarations ---
static class WindowFocus
{
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")] static extern bool AttachThreadInput(uint idAttach, uint idAttachTo, bool fAttach);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint processId);
    [DllImport("user32.dll")] static extern uint GetCurrentThreadId();
    [DllImport("user32.dll")] static extern bool IsIconic(IntPtr hWnd);
    [DllImport("user32.dll")] static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
    [DllImport("user32.dll")] static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);
    [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")] static extern void keybd_event(byte bVk, byte bScan, uint dwFlags, UIntPtr dwExtraInfo);

    delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

    const int SW_RESTORE = 9;
    const int SW_MINIMIZE = 6;
    const byte VK_MENU = 0x12; // Alt key
    const uint KEYEVENTF_EXTENDEDKEY = 0x0001;
    const uint KEYEVENTF_KEYUP = 0x0002;

    public static IntPtr FindMainWindowByPid(uint targetPid)
    {
        IntPtr found = IntPtr.Zero;
        EnumWindows((hWnd, _) =>
        {
            GetWindowThreadProcessId(hWnd, out uint pid);
            if (pid == targetPid && IsWindowVisible(hWnd))
            {
                found = hWnd;
                return false; // stop enumeration
            }
            return true;
        }, IntPtr.Zero);
        return found;
    }

    public static bool FocusWindow(IntPtr hwnd)
    {
        // Restore if minimized
        if (IsIconic(hwnd))
        {
            Console.WriteLine("  Window is minimized, restoring...");
            ShowWindow(hwnd, SW_RESTORE);
            System.Threading.Thread.Sleep(50);
        }

        // Strategy 1: Direct SetForegroundWindow
        if (SetForegroundWindow(hwnd))
        {
            System.Threading.Thread.Sleep(50);
            if (GetForegroundWindow() == hwnd)
            {
                Console.WriteLine("  Strategy 1 (direct): SUCCESS");
                return true;
            }
        }

        // Strategy 2: AttachThreadInput
        var fgHwnd = GetForegroundWindow();
        var fgTid = GetWindowThreadProcessId(fgHwnd, out _);
        var ourTid = GetCurrentThreadId();
        AttachThreadInput(ourTid, fgTid, true);
        SetForegroundWindow(hwnd);
        AttachThreadInput(ourTid, fgTid, false);
        System.Threading.Thread.Sleep(50);
        if (GetForegroundWindow() == hwnd)
        {
            Console.WriteLine("  Strategy 2 (AttachThreadInput): SUCCESS");
            return true;
        }

        // Strategy 3: Alt key hack
        keybd_event(VK_MENU, 0, KEYEVENTF_EXTENDEDKEY, UIntPtr.Zero);
        keybd_event(VK_MENU, 0, KEYEVENTF_KEYUP, UIntPtr.Zero);
        SetForegroundWindow(hwnd);
        System.Threading.Thread.Sleep(50);
        if (GetForegroundWindow() == hwnd)
        {
            Console.WriteLine("  Strategy 3 (Alt key hack): SUCCESS");
            return true;
        }

        // Strategy 4: Minimize then restore
        ShowWindow(hwnd, SW_MINIMIZE);
        System.Threading.Thread.Sleep(50);
        ShowWindow(hwnd, SW_RESTORE);
        System.Threading.Thread.Sleep(50);
        if (GetForegroundWindow() == hwnd)
        {
            Console.WriteLine("  Strategy 4 (minimize/restore): SUCCESS");
            return true;
        }

        Console.WriteLine("  All strategies attempted, window may only flash in taskbar");
        return false;
    }
}
