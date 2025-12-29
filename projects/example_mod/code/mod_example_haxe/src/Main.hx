// Haxe Mod Entry Point for example_mod
// example_mod 的 Haxe 模组入口点

package;

import souprune.sdk.DanmakuRegistry;
import souprune.sdk.IDanmakuBehavior;
import behaviors.SpiralHomingDanmaku;
import behaviors.WaveBurstDanmaku;
import behaviors.GravityDropDanmaku;

/**
 * Entry point for the Haxe mod.
 * Contains the registration logic and exported functions that the game engine will call.
 * 
 * Haxe 模组的入口点。
 * 包含游戏引擎将调用的注册逻辑和导出函数。
 */
class Main {
    private static var initialized:Bool = false;
    private static var behaviorIds:Array<String>;
    
    /**
     * Initialize the mod and register behaviors.
     */
    private static function ensureInitialized():Void {
        if (initialized) return;
        initialized = true;
        
        // Register danmaku behaviors
        // 注册弹幕行为
        DanmakuRegistry.register("spiral_homing", () -> new SpiralHomingDanmaku());
        DanmakuRegistry.register("wave_burst", () -> new WaveBurstDanmaku());
        DanmakuRegistry.register("gravity_drop", () -> new GravityDropDanmaku());
        
        behaviorIds = DanmakuRegistry.getIds();
        
        trace('[ModExample Haxe] Initialized with ${behaviorIds.length} danmaku behaviors');
    }
    
    static function main() {
        ensureInitialized();
        trace("Haxe mod loaded!");
    }
}

