// C# Mod Entry Point for example_mod
// example_mod 的 C# 模组入口点

using System.Runtime.InteropServices;
using Souprune.Sdk;
using ModExample.Behaviors;

namespace ModExample;

/// <summary>
/// Entry point for the C# mod.
/// Contains the exported functions that the game engine will call.
/// 
/// C# 模组的入口点。
/// 包含游戏引擎将调用的导出函数。
/// </summary>
public static class ModEntry
{
    private static bool _initialized;
    private static string[]? _behaviorIds;
    
    /// <summary>
    /// Initialize the mod and register behaviors.
    /// </summary>
    private static void EnsureInitialized()
    {
        if (_initialized) return;
        _initialized = true;
        
        // Register danmaku behaviors
        // 注册弹幕行为
        DanmakuRegistry.Register<SpiralHomingDanmaku>("spiral_homing");
        DanmakuRegistry.Register<WaveBurstDanmaku>("wave_burst");
        DanmakuRegistry.Register<GravityDropDanmaku>("gravity_drop");
        
        _behaviorIds = DanmakuRegistry.GetIds();
        
        Console.WriteLine($"[ModExample C#] Initialized with {_behaviorIds.Length} danmaku behaviors");
    }
    
    /// <summary>
    /// Get the number of exported danmaku algorithms.
    /// </summary>
    [UnmanagedCallersOnly(EntryPoint = "get_algorithm_count")]
    public static uint GetAlgorithmCount()
    {
        EnsureInitialized();
        return (uint)(_behaviorIds?.Length ?? 0);
    }
    
    /// <summary>
    /// Get the ID of an algorithm by index.
    /// </summary>
    [UnmanagedCallersOnly(EntryPoint = "get_algorithm_id")]
    public static nint GetAlgorithmId(uint index)
    {
        EnsureInitialized();
        if (_behaviorIds == null || index >= _behaviorIds.Length)
            return 0;
        
        var id = _behaviorIds[index];
        var ptr = Marshal.StringToCoTaskMemUTF8(id);
        return ptr;
    }
    
    /// <summary>
    /// Create a danmaku behavior instance by ID.
    /// </summary>
    [UnmanagedCallersOnly(EntryPoint = "create_danmaku")]
    public static NativeDanmakuInstance CreateDanmaku(nint idPtr)
    {
        EnsureInitialized();
        
        if (idPtr == 0)
            return default;
        
        var id = Marshal.PtrToStringUTF8(idPtr);
        if (string.IsNullOrEmpty(id))
            return default;
        
        if (DanmakuRegistry.TryCreate(id, out var handle, out var vtable))
        {
            Console.WriteLine($"[ModExample C#] Created behavior '{id}' with handle {handle}");
            return new NativeDanmakuInstance
            {
                Instance = handle,
                VTable = vtable
            };
        }
        
        Console.WriteLine($"[ModExample C#] Failed to create behavior '{id}'");
        return default;
    }
}
