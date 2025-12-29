package souprune;

import souprune.ffi.SoupruneApi;
import souprune.ffi.Vec2C;
import souprune.ffi.BulletOutputC;
import souprune.ffi.BulletContextC;

/**
 * Safe Haxe wrapper for the Souprune SDK.
 * Provides a more idiomatic Haxe API over the raw FFI bindings.
 */

/**
 * 2D Vector helper class.
 */
class Vec2 {
    public var x:Float;
    public var y:Float;
    
    public function new(x:Float = 0, y:Float = 0) {
        this.x = x;
        this.y = y;
    }
    
    public static function fromNative(native:Vec2C):Vec2 {
        return new Vec2(native.x, native.y);
    }
    
    public function toNative():Vec2C {
        return SoupruneApi.vec2cNew(x, y);
    }
    
    public function length():Float {
        return SoupruneApi.vec2cLength(toNative());
    }
    
    public function normalize():Vec2 {
        var result = SoupruneApi.vec2cNormalize(toNative());
        return fromNative(result);
    }
    
    public function toString():String {
        return '($x, $y)';
    }
}

/**
 * Bullet output wrapper.
 */
class BulletOutput {
    public var offsetX:Float;
    public var offsetY:Float;
    public var rotation:Float;
    
    public function new(offsetX:Float = 0, offsetY:Float = 0, rotation:Float = 0) {
        this.offsetX = offsetX;
        this.offsetY = offsetY;
        this.rotation = rotation;
    }
    
    public static function fromNative(native:BulletOutputC):BulletOutput {
        return new BulletOutput(native.offset_x, native.offset_y, native.rotation);
    }
    
    public function toNative():BulletOutputC {
        return SoupruneApi.danmakuOutputWithRotation(offsetX, offsetY, rotation);
    }
    
    public function toString():String {
        return 'BulletOutput(offset=($offsetX, $offsetY), rotation=$rotation)';
    }
}

/**
 * Bullet context wrapper (read-only).
 */
class BulletContext {
    private var native:cpp.ConstPointer<BulletContextC>;
    
    public function new(native:cpp.ConstPointer<BulletContextC>) {
        this.native = native;
    }
    
    public function getElapsed():Float {
        return SoupruneApi.bulletContextGetElapsed(native);
    }
    
    public function getDeltaTime():Float {
        return SoupruneApi.bulletContextGetDeltaTime(native);
    }
    
    public function getSpawnPos():Vec2 {
        return Vec2.fromNative(SoupruneApi.bulletContextGetSpawnPos(native));
    }
    
    public function getOffset():Vec2 {
        return Vec2.fromNative(SoupruneApi.bulletContextGetOffset(native));
    }
    
    public function getInitialAngle():Float {
        return SoupruneApi.bulletContextGetInitialAngle(native);
    }
    
    public function getInitialRadius():Float {
        return SoupruneApi.bulletContextGetInitialRadius(native);
    }
    
    public function getPlayerPos():Vec2 {
        return Vec2.fromNative(SoupruneApi.bulletContextGetPlayerPos(native));
    }
}
