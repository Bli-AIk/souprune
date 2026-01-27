// Gravity Drop Danmaku - Bullets that fall with gravity and bounce.
// 重力下落弹幕 - 受重力影响下落并反弹的弹幕。

using Souprune.Sdk;

namespace ModExample.Behaviors;

/// <summary>
/// Gravity Drop Danmaku - falls with gravity and bounces off the bottom.
/// 重力下落弹幕 - 受重力影响下落并在底部反弹。
/// 
/// Properties (from RON config):
/// - "gravity": gravity acceleration (default: 200.0)
/// - "bounce_damping": velocity retained after bounce 0-1 (default: 0.7)
/// </summary>
public class GravityDropDanmaku : IDanmakuBehavior
{
    // Configuration
    private float _initialVelocityX = 50f;
    private float _gravity = 200f;
    private float _bounceY = -80f;
    private float _bounceDamping = 0.7f;
    
    // State
    private float _velocityY;
    private float _posX;
    private float _posY;
    private int _bounceCount;
    
    public void OnEnter(in BulletContext ctx)
    {
        _velocityY = -100f; // Initial upward velocity
        _posX = 0;
        _posY = 0;
        _bounceCount = 0;
        
        // Randomize initial X velocity based on angle
        _initialVelocityX = MathF.Cos(ctx.InitialAngle) * 80f;
    }
    
    public BulletOutput OnUpdate(in BulletContext ctx)
    {
        // Apply gravity to Y velocity
        _velocityY += _gravity * ctx.DeltaTime;
        
        // Update positions
        _posX += _initialVelocityX * ctx.DeltaTime;
        _posY += _velocityY * ctx.DeltaTime;
        
        // Check for bounce (max 3 bounces)
        if (_posY > _bounceY && _bounceCount < 3 && _velocityY > 0)
        {
            _posY = _bounceY;
            _velocityY = -_velocityY * _bounceDamping;
            _bounceCount++;
        }
        
        // Rotation based on velocity direction
        var rotation = MathF.Atan2(_velocityY, _initialVelocityX);
        
        return BulletOutput.FromOffset(_posX, _posY).WithRotation(rotation);
    }
    
    public void OnExit() { }
}
