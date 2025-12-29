// Souprune SDK for Haxe - Native Export Infrastructure
// 用于 Haxe 的 Souprune SDK - 原生导出基础设施

package souprune.sdk;

import cpp.Callable;
import cpp.ConstPointer;
import cpp.Pointer;
import cpp.RawPointer;

/**
 * Registry for danmaku behaviors.
 * Register your behaviors using the register method, then they will be exported to the game.
 * 
 * 弹幕行为注册表。
 * 使用 register 方法注册你的行为，然后它们将被导出到游戏中。
 */
class DanmakuRegistry {
    private static var factories:Map<String, Void->IDanmakuBehavior> = new Map();
    private static var instances:Map<Int, IDanmakuBehavior> = new Map();
    private static var nextHandle:Int = 1;
    
    /**
     * Register a danmaku behavior factory.
     * 
     * 注册一个弹幕行为工厂。
     * @param id Unique identifier for this behavior (referenced in .danmaku.ron)
     * @param factory Factory function to create instances
     */
    public static function register(id:String, factory:Void->IDanmakuBehavior):Void {
        factories.set(id, factory);
    }
    
    /**
     * Get all registered behavior IDs.
     */
    public static function getIds():Array<String> {
        return [for (key in factories.keys()) key];
    }
    
    /**
     * Get the number of registered behaviors.
     */
    public static function getCount():Int {
        var count = 0;
        for (_ in factories.keys()) count++;
        return count;
    }
    
    /**
     * Try to create a behavior instance by ID.
     */
    public static function tryCreate(id:String):Null<{handle:Int, behavior:IDanmakuBehavior}> {
        var factory = factories.get(id);
        if (factory == null) return null;
        
        var behavior = factory();
        var handle = nextHandle++;
        instances.set(handle, behavior);
        return {handle: handle, behavior: behavior};
    }
    
    /**
     * Get a behavior instance by handle.
     */
    public static function getInstance(handle:Int):Null<IDanmakuBehavior> {
        return instances.get(handle);
    }
    
    /**
     * Destroy a behavior instance.
     */
    public static function destroyInstance(handle:Int):Void {
        instances.remove(handle);
    }
}

/**
 * Native VTable structure matching Rust's DanmakuVTable.
 * This is used internally for FFI - mod authors don't need to use this directly.
 */
@:structAccess
@:native("DanmakuVTable")
extern class NativeDanmakuVTable {
    public var on_enter:RawPointer<cpp.Void>;
    public var on_update:RawPointer<cpp.Void>;
    public var on_exit:RawPointer<cpp.Void>;
    public var destroy:RawPointer<cpp.Void>;
}

/**
 * Native DanmakuInstance structure matching Rust's DanmakuInstance.
 */
@:structAccess
@:native("DanmakuInstance")
extern class NativeDanmakuInstance {
    public var instance:RawPointer<cpp.Void>;
    public var vtable:NativeDanmakuVTable;
}

/**
 * Native BulletContext structure matching Rust's BulletContextC.
 */
@:structAccess
@:native("BULLETCONTEXTC")
extern class NativeBulletContextC {
    public var elapsed:cpp.Float32;
    public var delta_time:cpp.Float32;
    public var spawn_x:cpp.Float32;
    public var spawn_y:cpp.Float32;
    public var offset_x:cpp.Float32;
    public var offset_y:cpp.Float32;
    public var initial_angle:cpp.Float32;
    public var initial_radius:cpp.Float32;
    public var player_x:cpp.Float32;
    public var player_y:cpp.Float32;
    
    public function toManaged():BulletContext {
        return new BulletContext(
            elapsed, delta_time,
            new Vec2(spawn_x, spawn_y),
            new Vec2(offset_x, offset_y),
            initial_angle, initial_radius,
            new Vec2(player_x, player_y)
        );
    }
}

/**
 * Native BulletOutput structure matching Rust's BulletOutputC.
 */
@:structAccess
@:native("BULLETOUTPUTC")
extern class NativeBulletOutputC {
    public var offset_x:cpp.Float32;
    public var offset_y:cpp.Float32;
    public var rotation:cpp.Float32;
    
    public static function fromManaged(output:BulletOutput):NativeBulletOutputC {
        var native = untyped __cpp__("BULLETOUTPUTC{}");
        native.offset_x = output.offset.x;
        native.offset_y = output.offset.y;
        native.rotation = output.rotation;
        return native;
    }
}
