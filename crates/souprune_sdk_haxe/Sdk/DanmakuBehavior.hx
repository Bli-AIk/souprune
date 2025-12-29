// Souprune SDK for Haxe - Danmaku Behavior Interface
// 用于 Haxe 的 Souprune SDK - 弹幕行为接口

package souprune.sdk;

/**
 * Base interface for all danmaku (bullet pattern) behaviors.
 * Implement this interface to create custom bullet movement patterns.
 * 
 * 所有弹幕行为的基础接口。
 * 实现此接口以创建自定义弹幕移动模式。
 */
interface IDanmakuBehavior {
    /**
     * Called once when the bullet is spawned.
     * Use this to capture initial state (e.g., player position for aimed behaviors).
     * 
     * 弹幕生成时调用一次。
     * 用于捕获初始状态（例如：自机狙的玩家位置）。
     */
    function onEnter(ctx:BulletContext):Void;
    
    /**
     * Called every frame to compute bullet movement.
     * Returns the position offset for this frame.
     * 
     * 每帧调用以计算弹幕移动。
     * 返回本帧的位置偏移。
     */
    function onUpdate(ctx:BulletContext):BulletOutput;
    
    /**
     * Called when the bullet is despawned.
     * 
     * 弹幕销毁时调用。
     */
    function onExit():Void;
}

/**
 * Bullet context with all the information needed for behavior calculation.
 * 
 * 包含行为计算所需所有信息的弹幕上下文。
 */
class BulletContext {
    /** Time since bullet spawn (seconds) */
    public var elapsed:Float;
    
    /** Delta time for this frame */
    public var deltaTime:Float;
    
    /** Spawn center position */
    public var spawnPos:Vec2;
    
    /** Initial offset from spawn center */
    public var initialOffset:Vec2;
    
    /** Initial angle (radians) */
    public var initialAngle:Float;
    
    /** Initial radius (for circular patterns) */
    public var initialRadius:Float;
    
    /** Current player position (for aimed behaviors) */
    public var playerPos:Vec2;
    
    public function new(
        elapsed:Float, deltaTime:Float,
        spawnPos:Vec2, initialOffset:Vec2,
        initialAngle:Float, initialRadius:Float,
        playerPos:Vec2
    ) {
        this.elapsed = elapsed;
        this.deltaTime = deltaTime;
        this.spawnPos = spawnPos;
        this.initialOffset = initialOffset;
        this.initialAngle = initialAngle;
        this.initialRadius = initialRadius;
        this.playerPos = playerPos;
    }
    
    /**
     * Get current position (spawn + offset).
     * 获取当前位置（生成点 + 偏移）。
     */
    public function getSpawnPosition():Vec2 {
        return new Vec2(spawnPos.x + initialOffset.x, spawnPos.y + initialOffset.y);
    }
}

/**
 * Output from a danmaku behavior's onUpdate method.
 * 
 * 弹幕行为 onUpdate 方法的输出。
 */
class BulletOutput {
    /** Position offset */
    public var offset:Vec2;
    
    /** Rotation delta (radians) */
    public var rotation:Float;
    
    public function new(?offset:Vec2, rotation:Float = 0.0) {
        this.offset = offset != null ? offset : Vec2.zero();
        this.rotation = rotation;
    }
    
    public static function zero():BulletOutput {
        return new BulletOutput();
    }
    
    public static function fromOffset(x:Float, y:Float):BulletOutput {
        return new BulletOutput(new Vec2(x, y));
    }
    
    public function withRotation(r:Float):BulletOutput {
        this.rotation = r;
        return this;
    }
}

/**
 * Simple 2D vector for SDK use.
 * 
 * SDK 使用的简单二维向量。
 */
class Vec2 {
    public var x:Float;
    public var y:Float;
    
    public function new(x:Float = 0, y:Float = 0) {
        this.x = x;
        this.y = y;
    }
    
    public static function zero():Vec2 {
        return new Vec2(0, 0);
    }
    
    public function length():Float {
        return Math.sqrt(x * x + y * y);
    }
    
    public function normalize():Vec2 {
        var len = length();
        if (len > 0.0001) {
            return new Vec2(x / len, y / len);
        }
        return zero();
    }
    
    public function add(other:Vec2):Vec2 {
        return new Vec2(x + other.x, y + other.y);
    }
    
    public function sub(other:Vec2):Vec2 {
        return new Vec2(x - other.x, y - other.y);
    }
    
    public function scale(s:Float):Vec2 {
        return new Vec2(x * s, y * s);
    }
    
    /**
     * Create a unit vector from an angle (radians).
     * 从角度（弧度）创建单位向量。
     */
    public static function fromAngle(radians:Float):Vec2 {
        return new Vec2(Math.cos(radians), Math.sin(radians));
    }
}
