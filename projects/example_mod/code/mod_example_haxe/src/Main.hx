package;

import souprune.ffi.SoupruneApi;
import souprune.ffi.Vec2C;
import souprune.ffi.BulletOutputC;

/**
 * Hello World Mod for Souprune - Haxe version
 * 
 * This demonstrates basic FFI usage with the Souprune API.
 */
class Main {
    static function main() {
        trace("Hello from Haxe Mod!");
        
        // Example: Create a Vec2C
        var vec = SoupruneApi.vec2cNew(3.0, 4.0);
        trace('Created vector: (${vec.x}, ${vec.y})');
        
        // Example: Get vector length
        var length = SoupruneApi.vec2cLength(vec);
        trace('Vector length: $length');
        
        // Example: Normalize the vector
        var normalized = SoupruneApi.vec2cNormalize(vec);
        trace('Normalized: (${normalized.x}, ${normalized.y})');
        
        // Example: Create bullet output
        var output = SoupruneApi.danmakuOutputNew(10.0, 20.0);
        trace('Bullet output: offset=(${output.offset_x}, ${output.offset_y}), rotation=${output.rotation}');
        
        // Example: Create bullet output with rotation
        var rotatedOutput = SoupruneApi.danmakuOutputWithRotation(5.0, 10.0, 1.57);
        trace('Rotated output: offset=(${rotatedOutput.offset_x}, ${rotatedOutput.offset_y}), rotation=${rotatedOutput.rotation}');
        
        trace("Haxe mod initialization complete!");
    }
}
