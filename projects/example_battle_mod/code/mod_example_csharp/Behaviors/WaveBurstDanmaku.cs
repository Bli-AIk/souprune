// Wave Burst Danmaku - Bullets that move in a sine wave pattern with burst acceleration.
// 波动爆发弹幕 - 以正弦波模式移动并带有爆发加速的弹幕。

using Souprune.Sdk;

namespace ModExample.Behaviors;

/// <summary>
/// Wave Burst Danmaku - moves in sine wave pattern with initial burst speed.
/// 波动爆发弹幕 - 以正弦波模式移动，初始有爆发速度。
/// 
/// Properties (from RON config):
/// - "base_speed": base movement speed (default: 120.0)
/// - "wave_amplitude": wave height in pixels (default: 30.0)
/// - "wave_frequency": waves per second (default: 4.0)
/// - "burst_time": burst duration in seconds (default: 0.8)
/// - "burst_multiplier": initial speed multiplier (default: 2.5)
/// </summary>
public class WaveBurstDanmaku : IDanmakuBehavior
{
    // Configuration
    private float _baseSpeed = 120f;
    private float _waveAmplitude = 30f;
    private float _waveFrequency = 4f;
    private float _burstTime = 0.8f;
    private float _burstMultiplier = 2.5f;
    
    // State
    private Vec2 _direction;
    private Vec2 _perpendicular;
    
    public void OnEnter(in BulletContext ctx)
    {
        // Use initial angle to determine direction
        _direction = Vec2.FromAngle(ctx.InitialAngle);
        _perpendicular = new Vec2(-_direction.Y, _direction.X);
    }
    
    public BulletOutput OnUpdate(in BulletContext ctx)
    {
        // Calculate burst multiplier (ease out)
        var speedMultiplier = 1f;
        if (ctx.Elapsed < _burstTime)
        {
            var t = ctx.Elapsed / _burstTime;
            speedMultiplier = 1f + (_burstMultiplier - 1f) * (1f - t * t);
        }
        
        // Calculate linear movement
        var linearDist = _baseSpeed * ctx.Elapsed * speedMultiplier;
        
        // Calculate wave offset
        var wave = MathF.Sin(ctx.Elapsed * _waveFrequency * MathF.PI * 2f) * _waveAmplitude;
        
        // Combine movements
        var offset = _direction * linearDist + _perpendicular * wave;
        
        // Rotation follows wave
        var rotation = MathF.Sin(ctx.Elapsed * _waveFrequency * MathF.PI * 2f) * 0.3f;
        
        return BulletOutput.FromOffset(offset).WithRotation(rotation);
    }
    
    public void OnExit() { }
}
