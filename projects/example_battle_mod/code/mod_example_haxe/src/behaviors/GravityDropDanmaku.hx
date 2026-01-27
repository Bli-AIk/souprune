// Gravity Drop Danmaku - Bullets that fall with gravity and bounce.
// 重力下落弹幕 - 受重力影响下落并反弹的弹幕。

package behaviors;

import souprune.sdk.IDanmakuBehavior;
import souprune.sdk.BulletContext;
import souprune.sdk.BulletOutput;
import souprune.sdk.Vec2;

/**
 * Gravity Drop Danmaku - falls with gravity and bounces off the bottom.
 * 重力下落弹幕 - 受重力影响下落并在底部反弹。
 * 
 * Properties (from RON config):
 * - "gravity": gravity acceleration (default: 200.0)
 * - "bounce_damping": velocity retained after bounce 0-1 (default: 0.7)
 */
class GravityDropDanmaku implements IDanmakuBehavior {
    // Configuration
    private var initialVelocityX:Float = 50.0;
    private var gravity:Float = 200.0;
    private var bounceY:Float = -80.0;
    private var bounceDamping:Float = 0.7;
    
    // State
    private var velocityY:Float = 0.0;
    private var posX:Float = 0.0;
    private var posY:Float = 0.0;
    private var bounceCount:Int = 0;
    
    public function new() { }
    
    public function onEnter(ctx:BulletContext):Void {
        velocityY = -100.0; // Initial upward velocity
        posX = 0;
        posY = 0;
        bounceCount = 0;
        
        // Randomize initial X velocity based on angle
        initialVelocityX = Math.cos(ctx.initialAngle) * 80.0;
    }
    
    public function onUpdate(ctx:BulletContext):BulletOutput {
        // Apply gravity to Y velocity
        velocityY += gravity * ctx.deltaTime;
        
        // Update positions
        posX += initialVelocityX * ctx.deltaTime;
        posY += velocityY * ctx.deltaTime;
        
        // Check for bounce (max 3 bounces)
        if (posY > bounceY && bounceCount < 3 && velocityY > 0) {
            posY = bounceY;
            velocityY = -velocityY * bounceDamping;
            bounceCount++;
        }
        
        // Rotation based on velocity direction
        var rotation = Math.atan2(velocityY, initialVelocityX);
        
        return BulletOutput.fromOffset(posX, posY).withRotation(rotation);
    }
    
    public function onExit():Void { }
}
