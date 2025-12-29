// Wave Burst Danmaku - Bullets that move in a sine wave pattern with burst acceleration.
// 波动爆发弹幕 - 以正弦波模式移动并带有爆发加速的弹幕。

package behaviors;

import souprune.sdk.IDanmakuBehavior;
import souprune.sdk.BulletContext;
import souprune.sdk.BulletOutput;
import souprune.sdk.Vec2;

/**
 * Wave Burst Danmaku - moves in sine wave pattern with initial burst speed.
 * 波动爆发弹幕 - 以正弦波模式移动，初始有爆发速度。
 * 
 * Properties (from RON config):
 * - "base_speed": base movement speed (default: 120.0)
 * - "wave_amplitude": wave height in pixels (default: 30.0)
 * - "wave_frequency": waves per second (default: 4.0)
 * - "burst_time": burst duration in seconds (default: 0.8)
 * - "burst_multiplier": initial speed multiplier (default: 2.5)
 */
class WaveBurstDanmaku implements IDanmakuBehavior {
    // Configuration
    private var baseSpeed:Float = 120.0;
    private var waveAmplitude:Float = 30.0;
    private var waveFrequency:Float = 4.0;
    private var burstTime:Float = 0.8;
    private var burstMultiplier:Float = 2.5;
    
    // State
    private var direction:Vec2;
    private var perpendicular:Vec2;
    
    public function new() {
        direction = Vec2.zero();
        perpendicular = Vec2.zero();
    }
    
    public function onEnter(ctx:BulletContext):Void {
        // Use initial angle to determine direction
        direction = Vec2.fromAngle(ctx.initialAngle);
        perpendicular = new Vec2(-direction.y, direction.x);
    }
    
    public function onUpdate(ctx:BulletContext):BulletOutput {
        // Calculate burst multiplier (ease out)
        var speedMultiplier = 1.0;
        if (ctx.elapsed < burstTime) {
            var t = ctx.elapsed / burstTime;
            speedMultiplier = 1.0 + (burstMultiplier - 1.0) * (1.0 - t * t);
        }
        
        // Calculate linear movement
        var linearDist = baseSpeed * ctx.elapsed * speedMultiplier;
        
        // Calculate wave offset
        var wave = Math.sin(ctx.elapsed * waveFrequency * Math.PI * 2.0) * waveAmplitude;
        
        // Combine movements
        var offset = direction.scale(linearDist).add(perpendicular.scale(wave));
        
        // Rotation follows wave
        var rotation = Math.sin(ctx.elapsed * waveFrequency * Math.PI * 2.0) * 0.3;
        
        return BulletOutput.fromOffset(offset.x, offset.y).withRotation(rotation);
    }
    
    public function onExit():Void { }
}
