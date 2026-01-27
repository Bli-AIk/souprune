// Spiral Homing Danmaku - A bullet that spirals outward while slowly tracking the player.
// 螺旋追踪弹幕 - 一个向外螺旋运动同时缓慢追踪玩家的弹幕。

using Souprune.Sdk;

namespace ModExample.Behaviors;

/// <summary>
/// Spiral Homing Danmaku - spirals outward while slowly homing toward the player.
/// 螺旋追踪弹幕 - 向外螺旋运动同时缓慢追踪玩家。
/// 
/// Properties (from RON config):
/// - "spiral_speed": outward spiral speed (default: 80.0)
/// - "angular_velocity": rotation speed in rad/s (default: 3.0)
/// - "homing_strength": how strongly it tracks player 0-1 (default: 0.5)
/// - "homing_delay": delay before homing starts in seconds (default: 0.5)
/// </summary>
public class SpiralHomingDanmaku : IDanmakuBehavior
{
    // Configuration
    private float _spiralSpeed = 80f;
    private float _angularVelocity = 3.0f;
    private float _homingStrength = 0.5f;
    private float _homingDelay = 0.5f;
    
    // Cached state from OnEnter
    private Vec2 _capturedDirection;
    
    // Runtime state
    private float _currentAngle;
    private float _accumulatedRadius;
    
    public void OnEnter(in BulletContext ctx)
    {
        // Calculate initial direction towards player
        var toPlayer = ctx.PlayerPos - ctx.SpawnPosition;
        _capturedDirection = toPlayer.Length() > 0.001f 
            ? toPlayer.Normalize() 
            : new Vec2(0, -1);
            
        // Initialize angle from spawn angle
        _currentAngle = ctx.InitialAngle;
        _accumulatedRadius = ctx.InitialRadius;
    }
    
    public BulletOutput OnUpdate(in BulletContext ctx)
    {
        // Update spiral angle
        _currentAngle += _angularVelocity * ctx.DeltaTime;
        
        // Increase radius over time (spiral outward)
        _accumulatedRadius += _spiralSpeed * ctx.DeltaTime;
        
        // Calculate base spiral position
        var spiralX = MathF.Cos(_currentAngle) * _accumulatedRadius;
        var spiralY = MathF.Sin(_currentAngle) * _accumulatedRadius;
        var spiralOffset = new Vec2(spiralX, spiralY);
        
        // After delay, blend in homing towards current player position
        if (ctx.Elapsed > _homingDelay)
        {
            var homingT = MathF.Min((ctx.Elapsed - _homingDelay) * _homingStrength, 1f);
            
            // Calculate direction to current player
            var currentToPlayer = ctx.PlayerPos - ctx.SpawnPosition;
            var targetDirection = currentToPlayer.Length() > 0.001f 
                ? currentToPlayer.Normalize() 
                : _capturedDirection;
            
            // Blend captured direction towards current player
            var blendedDirection = Lerp(_capturedDirection, targetDirection, homingT * 0.3f);
            
            // Add a drift component in the homing direction
            var driftMagnitude = ctx.Elapsed * 30f * homingT;
            spiralOffset = spiralOffset + blendedDirection * driftMagnitude;
        }
        
        // Apply visual rotation based on movement
        var rotation = _currentAngle * 0.5f;
        
        return BulletOutput.FromOffset(spiralOffset).WithRotation(rotation);
    }
    
    public void OnExit() { }
    
    private static Vec2 Lerp(Vec2 a, Vec2 b, float t)
    {
        return new Vec2(
            a.X + (b.X - a.X) * t,
            a.Y + (b.Y - a.Y) * t
        );
    }
}
