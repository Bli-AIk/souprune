use super::*;

/// System to update bullet lifetime timers.
///
/// 更新弹幕生命周期计时器的系统。
pub fn update_bullet_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut BulletLifetime), With<Bullet>>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.timer.tick(time.delta());

        if lifetime.timer.is_finished() {
            commands.entity(entity).insert(DespawnBullet);
        }
    }
}

/// System to cleanup bullets marked for despawn.
/// Calls WASM on_exit for any active danmaku before despawning.
pub fn cleanup_dead_bullets(
    mut commands: Commands,
    mut query: Query<
        (Entity, Option<&mut ActiveDanmakuStack>),
        (With<Bullet>, With<DespawnBullet>),
    >,
    mut loaded_mods: NonSendMut<LoadedMods>,
) {
    for (entity, danmaku_stack) in query.iter_mut() {
        if let Some(mut stack) = danmaku_stack {
            stack.call_on_exit_all(&mut loaded_mods);
        }
        commands.entity(entity).despawn();
    }
}

/// System to despawn bullet containers that have no remaining children.
pub fn cleanup_empty_containers(
    mut commands: Commands,
    container_query: Query<(Entity, &Children), With<BulletContainer>>,
) {
    for (entity, children) in container_query.iter() {
        if children.is_empty() {
            commands.entity(entity).despawn();
        }
    }
}
