// Souprune SDK for C# - Danmaku Behavior Interface
// 用于 C# 的 Souprune SDK - 弹幕行为接口

namespace Souprune.Sdk;

/// <summary>
/// Base interface for all danmaku (bullet pattern) behaviors.
/// Implement this interface to create custom bullet movement patterns.
/// 
/// 所有弹幕行为的基础接口。
/// 实现此接口以创建自定义弹幕移动模式。
/// </summary>
public interface IDanmakuBehavior
{
    /// <summary>
    /// Called once when the bullet is spawned.
    /// Use this to capture initial state (e.g., player position for aimed behaviors).
    /// 
    /// 弹幕生成时调用一次。
    /// 用于捕获初始状态（例如：自机狙的玩家位置）。
    /// </summary>
    void OnEnter(in BulletContext ctx);
    
    /// <summary>
    /// Called every frame to compute bullet movement.
    /// Returns the position offset for this frame.
    /// 
    /// 每帧调用以计算弹幕移动。
    /// 返回本帧的位置偏移。
    /// </summary>
    BulletOutput OnUpdate(in BulletContext ctx);
    
    /// <summary>
    /// Called when the bullet is despawned.
    /// 
    /// 弹幕销毁时调用。
    /// </summary>
    void OnExit();
}

/// <summary>
/// Bullet context with all the information needed for behavior calculation.
/// This is a managed wrapper around the native BulletContextC.
/// 
/// 包含行为计算所需所有信息的弹幕上下文。
/// 这是原生 BulletContextC 的托管包装器。
/// </summary>
public readonly struct BulletContext
{
    /// <summary>Time since bullet spawn (seconds)</summary>
    public readonly float Elapsed;
    
    /// <summary>Delta time for this frame</summary>
    public readonly float DeltaTime;
    
    /// <summary>Spawn center position</summary>
    public readonly Vec2 SpawnPos;
    
    /// <summary>Initial offset from spawn center</summary>
    public readonly Vec2 InitialOffset;
    
    /// <summary>Initial angle (radians)</summary>
    public readonly float InitialAngle;
    
    /// <summary>Initial radius (for circular patterns)</summary>
    public readonly float InitialRadius;
    
    /// <summary>Current player position (for aimed behaviors)</summary>
    public readonly Vec2 PlayerPos;
    
    public BulletContext(
        float elapsed, float deltaTime,
        Vec2 spawnPos, Vec2 initialOffset,
        float initialAngle, float initialRadius,
        Vec2 playerPos)
    {
        Elapsed = elapsed;
        DeltaTime = deltaTime;
        SpawnPos = spawnPos;
        InitialOffset = initialOffset;
        InitialAngle = initialAngle;
        InitialRadius = initialRadius;
        PlayerPos = playerPos;
    }
    
    /// <summary>
    /// Get current position (spawn + offset).
    /// 获取当前位置（生成点 + 偏移）。
    /// </summary>
    public Vec2 SpawnPosition => SpawnPos + InitialOffset;
}

/// <summary>
/// Output from a danmaku behavior's OnUpdate method.
/// 
/// 弹幕行为 OnUpdate 方法的输出。
/// </summary>
public struct BulletOutput
{
    /// <summary>Position offset</summary>
    public Vec2 Offset;
    
    /// <summary>Rotation delta (radians)</summary>
    public float Rotation;
    
    public static readonly BulletOutput Zero = new() { Offset = Vec2.Zero, Rotation = 0f };
    
    public static BulletOutput FromOffset(float x, float y) => new() { Offset = new Vec2(x, y), Rotation = 0f };
    
    public static BulletOutput FromOffset(Vec2 offset) => new() { Offset = offset, Rotation = 0f };
    
    public BulletOutput WithRotation(float rotation)
    {
        Rotation = rotation;
        return this;
    }
}

/// <summary>
/// Simple 2D vector for SDK use.
/// 
/// SDK 使用的简单二维向量。
/// </summary>
public struct Vec2
{
    public float X;
    public float Y;
    
    public static readonly Vec2 Zero = new(0, 0);
    
    public Vec2(float x, float y) { X = x; Y = y; }
    
    public readonly float Length() => MathF.Sqrt(X * X + Y * Y);
    
    public readonly Vec2 Normalize()
    {
        var len = Length();
        return len > 0.0001f ? new Vec2(X / len, Y / len) : Zero;
    }
    
    public static Vec2 operator +(Vec2 a, Vec2 b) => new(a.X + b.X, a.Y + b.Y);
    public static Vec2 operator -(Vec2 a, Vec2 b) => new(a.X - b.X, a.Y - b.Y);
    public static Vec2 operator *(Vec2 v, float s) => new(v.X * s, v.Y * s);
    public static Vec2 operator *(float s, Vec2 v) => new(v.X * s, v.Y * s);
    
    /// <summary>
    /// Create a unit vector from an angle (radians).
    /// 从角度（弧度）创建单位向量。
    /// </summary>
    public static Vec2 FromAngle(float radians) => new(MathF.Cos(radians), MathF.Sin(radians));
}
