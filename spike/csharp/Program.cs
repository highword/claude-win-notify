using System;

if (args.Length > 0 && args[0] == "--version")
{
    Console.WriteLine("CSharpSpike v0.1");
    return;
}

Console.WriteLine("CSharpSpike: Hello from NativeAOT!");
Console.WriteLine($"  Runtime: {System.Runtime.InteropServices.RuntimeInformation.FrameworkDescription}");
Console.WriteLine($"  OS: {Environment.OSVersion}");
