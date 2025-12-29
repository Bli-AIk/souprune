

#ifndef interoptopus_generated
#define interoptopus_generated

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>
#include <sys/types.h>




///  Bullet context for C-ABI (simplified for bindings).
typedef struct BULLETCONTEXTC
    {
    float elapsed;
    float delta_time;
    float spawn_x;
    float spawn_y;
    float offset_x;
    float offset_y;
    float initial_angle;
    float initial_radius;
    float player_x;
    float player_y;
    } BULLETCONTEXTC;

///  Bullet output for C-ABI.
typedef struct BULLETOUTPUTC
    {
    float offset_x;
    float offset_y;
    float rotation;
    } BULLETOUTPUTC;

///  C-compatible 2D vector for FFI.
typedef struct VEC2C
    {
    float x;
    float y;
    } VEC2C;


///  Create a new BulletOutputC with position offset.
BULLETOUTPUTC danmaku_output_new(float OFFSET_X, float OFFSET_Y);

///  Create a BulletOutputC with rotation.
BULLETOUTPUTC danmaku_output_with_rotation(float OFFSET_X, float OFFSET_Y, float ROTATION);

///  Create a new Vec2C.
VEC2C vec2c_new(float X, float Y);

///  Get length of Vec2C.
float vec2c_length(VEC2C V);

///  Normalize a Vec2C.
VEC2C vec2c_normalize(VEC2C V);

///  Get elapsed time from bullet context.
float bullet_context_get_elapsed(const BULLETCONTEXTC* CTX);

///  Get delta time from bullet context.
float bullet_context_get_delta_time(const BULLETCONTEXTC* CTX);

///  Get spawn position from bullet context.
VEC2C bullet_context_get_spawn_pos(const BULLETCONTEXTC* CTX);

///  Get initial offset from bullet context.
VEC2C bullet_context_get_offset(const BULLETCONTEXTC* CTX);

///  Get initial angle from bullet context.
float bullet_context_get_initial_angle(const BULLETCONTEXTC* CTX);

///  Get initial radius from bullet context.
float bullet_context_get_initial_radius(const BULLETCONTEXTC* CTX);

///  Get player position from bullet context.
VEC2C bullet_context_get_player_pos(const BULLETCONTEXTC* CTX);


#ifdef __cplusplus
}
#endif

#endif /* interoptopus_generated */
