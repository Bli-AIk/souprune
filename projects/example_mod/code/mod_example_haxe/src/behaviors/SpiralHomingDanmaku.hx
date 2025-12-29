// Spiral Homing Danmaku - A bullet that spirals outward while slowly tracking the player.
// 螺旋追踪弹幕 - 一个向外螺旋运动同时缓慢追踪玩家的弹幕。

package behaviors;

import souprune.sdk.IDanmakuBehavior;
import souprune.sdk.BulletContext;
import souprune.sdk.BulletOutput;
import souprune.sdk.Vec2;

/**
 * Spiral Homing Danmaku - spirals outward while slowly homing toward the player.
 * 螺旋追踪弹幕 - 向外螺旋运动同时缓慢追踪玩家。
 * 
 * Properties (from RON config):
 * - "spiral_speed": outward spiral speed (default: 80.0)
 * - "angular_velocity": rotation speed in rad/s (default: 3.0)
 * - "homing_strength": how strongly it tracks player 0-1 (default: 0.5)
 * - "homing_delay": delay before homing starts in seconds (default: 0.5)
 */
class SpiralHomingDanmaku implements IDanmakuBehavior {
    // Configuration
    private var spiralSpeed:Float = 80.0;
    private var angularVelocity:Float = 3.0;
    private var homingStrength:Float = 0.5;
    private var homingDelay:Float = 0.5;
    
    // Cached state from onEnter
    private var capturedDirection:Vec2;
    
    // Runtime state
    private var currentAngle:Float = 0.0;
    private var accumulatedRadius:Float = 0.0;
    
    public function new() {
        capturedDirection = Vec2.zero();
    }
    
    public function onEnter(ctx:BulletContext):Void {
        // Calculate initial direction towards player
        var spawnPos = ctx.getSpawnPosition();
        var toPlayer = ctx.playerPos.sub(spawnPos);
        capturedDirection = toPlayer.length() > 0.001 
            ? toPlayer.normalize() 
            : new Vec2(0, -1);
            
        // Initialize angle from spawn angle
        currentAngle = ctx.initialAngle;
        accumulatedRadius = ctx.initialRadius;
    }
    
    public function onUpdate(ctx:BulletContext):BulletOutput {
        // Update spiral angle
        currentAngle += angularVelocity * ctx.deltaTime;
        
        // Increase radius over time (spiral outward)
        accumulatedRadius += spiralSpeed * ctx.deltaTime;
        
        // Calculate base spiral position
        var spiralX = Math.cos(currentAngle) * accumulatedRadius;
        var spiralY = Math.sin(currentAngle) * accumulatedRadius;
        var spiralOffset = new Vec2(spiralX, spiralY);
        
        // After delay, blend in homing towards current player position
        if (ctx.elapsed > homingDelay) {
            var homingT = Math.min((ctx.elapsed - homingDelay) * homingStrength, 1.0);
            
            // Calculate direction to current player
            var spawnPos = ctx.getSpawnPosition();
            var currentToPlayer = ctx.playerPos.sub(spawnPos);
            var targetDirection = currentToPlayer.length() > 0.001 
                ? currentToPlayer.normalize() 
                : capturedDirection;
            
            // Blend captured direction towards current player
            var blendedDirection = lerp(capturedDirection, targetDirection, homingT * 0.3);
            
            // Add a drift component in the homing direction
            var driftMagnitude = ctx.elapsed * 30.0 * homingT;
            spiralOffset = spiralOffset.add(blendedDirection.scale(driftMagnitude));
        }
        
        // Apply visual rotation based on movement
        var rotation = currentAngle * 0.5;
        
        return BulletOutput.fromOffset(spiralOffset.x, spiralOffset.y).withRotation(rotation);
    }
    
    public function onExit():Void { }
    
    private static function lerp(a:Vec2, b:Vec2, t:Float):Vec2 {
        return new Vec2(
            a.x + (b.x - a.x) * t,
            a.y + (b.y - a.y) * t
        );
    }
}
