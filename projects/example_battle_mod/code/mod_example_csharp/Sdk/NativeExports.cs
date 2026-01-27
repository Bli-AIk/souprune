// Souprune SDK for C# - Native Export Infrastructure
// 用于 C# 的 Souprune SDK - 原生导出基础设施

using System.Runtime.InteropServices;
using System.Runtime.CompilerServices;

namespace Souprune.Sdk;

/// <summary>
/// Registry for danmaku behaviors.
/// Register your behaviors using the Register method, then they will be exported to the game.
/// 
/// 弹幕行为注册表。
/// 使用 Register 方法注册你的行为，然后它们将被导出到游戏中。
/// </summary>
public static class DanmakuRegistry
{
    private static readonly Dictionary<string, Func<IDanmakuBehavior>> _factories = new();
    private static readonly Dictionary<nint, IDanmakuBehavior> _instances = new();
    private static nint _nextHandle = 1;
    
    /// <summary>
    /// Register a danmaku behavior factory.
    /// 
    /// 注册一个弹幕行为工厂。
    /// </summary>
    /// <param name="id">Unique identifier for this behavior (referenced in .performance.ron)</param>
    /// <param name="factory">Factory function to create instances</param>
    public static void Register(string id, Func<IDanmakuBehavior> factory)
    {
        _factories[id] = factory;
    }
    
    /// <summary>
    /// Register a danmaku behavior type using its default constructor.
    /// 
    /// 使用默认构造函数注册一个弹幕行为类型。
    /// </summary>
    public static void Register<T>(string id) where T : IDanmakuBehavior, new()
    {
        _factories[id] = () => new T();
    }
    
    internal static string[] GetIds() => _factories.Keys.ToArray();
    
    internal static bool TryCreate(string id, out nint handle, out NativeDanmakuVTable vtable)
    {
        if (_factories.TryGetValue(id, out var factory))
        {
            var instance = factory();
            handle = _nextHandle++;
            _instances[handle] = instance;
            vtable = NativeDanmakuVTable.Create();
            return true;
        }
        handle = 0;
        vtable = default;
        return false;
    }
    
    internal static IDanmakuBehavior? GetInstance(nint handle)
    {
        return _instances.TryGetValue(handle, out var instance) ? instance : null;
    }
    
    internal static void DestroyInstance(nint handle)
    {
        _instances.Remove(handle);
    }
}

/// <summary>
/// Native VTable structure matching Rust's DanmakuVTable.
/// </summary>
[StructLayout(LayoutKind.Sequential)]
internal struct NativeDanmakuVTable
{
    public nint OnEnter;
    public nint OnUpdate;
    public nint OnExit;
    public nint Destroy;
    
    public static NativeDanmakuVTable Create() => new()
    {
        OnEnter = Marshal.GetFunctionPointerForDelegate(NativeCallbacks.OnEnterDelegate),
        OnUpdate = Marshal.GetFunctionPointerForDelegate(NativeCallbacks.OnUpdateDelegate),
        OnExit = Marshal.GetFunctionPointerForDelegate(NativeCallbacks.OnExitDelegate),
        Destroy = Marshal.GetFunctionPointerForDelegate(NativeCallbacks.DestroyDelegate),
    };
}

/// <summary>
/// Native DanmakuInstance structure matching Rust's DanmakuInstance.
/// </summary>
[StructLayout(LayoutKind.Sequential)]
internal struct NativeDanmakuInstance
{
    public nint Instance;
    public NativeDanmakuVTable VTable;
}

/// <summary>
/// Native BulletContext structure matching Rust's BulletContextC.
/// </summary>
[StructLayout(LayoutKind.Sequential)]
internal struct NativeBulletContextC
{
    public float Elapsed;
    public float DeltaTime;
    public float SpawnX;
    public float SpawnY;
    public float OffsetX;
    public float OffsetY;
    public float InitialAngle;
    public float InitialRadius;
    public float PlayerX;
    public float PlayerY;
    public nint Props;
    public nuint PropsLen;
    public nint Params;
    public nuint ParamsLen;
    
    public BulletContext ToManaged() => new(
        Elapsed, DeltaTime,
        new Vec2(SpawnX, SpawnY),
        new Vec2(OffsetX, OffsetY),
        InitialAngle, InitialRadius,
        new Vec2(PlayerX, PlayerY)
    );
}

/// <summary>
/// Native BulletOutput structure matching Rust's BulletOutputC.
/// </summary>
[StructLayout(LayoutKind.Sequential)]
internal struct NativeBulletOutputC
{
    public float OffsetX;
    public float OffsetY;
    public float Rotation;
    
    public static NativeBulletOutputC FromManaged(BulletOutput output) => new()
    {
        OffsetX = output.Offset.X,
        OffsetY = output.Offset.Y,
        Rotation = output.Rotation
    };
}

/// <summary>
/// Native callback implementations.
/// </summary>
internal static class NativeCallbacks
{
    // Keep delegates alive to prevent GC
    internal static readonly OnEnterFn OnEnterDelegate = OnEnterImpl;
    internal static readonly OnUpdateFn OnUpdateDelegate = OnUpdateImpl;
    internal static readonly OnExitFn OnExitDelegate = OnExitImpl;
    internal static readonly DestroyFn DestroyDelegate = DestroyImpl;
    
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    internal delegate void OnEnterFn(nint instance, nint context);
    
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    internal delegate NativeBulletOutputC OnUpdateFn(nint instance, nint context);
    
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    internal delegate void OnExitFn(nint instance);
    
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    internal delegate void DestroyFn(nint instance);
    
    private static void OnEnterImpl(nint instance, nint contextPtr)
    {
        var behavior = DanmakuRegistry.GetInstance(instance);
        if (behavior == null) return;
        
        var nativeCtx = Marshal.PtrToStructure<NativeBulletContextC>(contextPtr);
        behavior.OnEnter(nativeCtx.ToManaged());
    }
    
    private static NativeBulletOutputC OnUpdateImpl(nint instance, nint contextPtr)
    {
        var behavior = DanmakuRegistry.GetInstance(instance);
        if (behavior == null) return default;
        
        var nativeCtx = Marshal.PtrToStructure<NativeBulletContextC>(contextPtr);
        var output = behavior.OnUpdate(nativeCtx.ToManaged());
        return NativeBulletOutputC.FromManaged(output);
    }
    
    private static void OnExitImpl(nint instance)
    {
        DanmakuRegistry.GetInstance(instance)?.OnExit();
    }
    
    private static void DestroyImpl(nint instance)
    {
        DanmakuRegistry.DestroyInstance(instance);
    }
}
