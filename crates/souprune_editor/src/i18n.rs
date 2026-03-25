//! # Internationalization (i18n)
//!
//! Editor localization support.

use bevy_workbench::i18n::{I18n, Locale};

mod locale_sources;

use self::locale_sources::{EN_FTL, ZH_CN_FTL};

/// 将编辑器的本地化字符串注册到 bevy_workbench I18n 系统。
pub fn register_editor_i18n(i18n: &mut I18n) {
    i18n.add_custom_source(Locale::En, EN_FTL);
    i18n.add_custom_source(Locale::ZhCn, ZH_CN_FTL);
}

/// Translate a message ID using the I18n resource from the given World.
pub fn t(world: &bevy::prelude::World, id: &str) -> String {
    world
        .get_resource::<I18n>()
        .map_or_else(|| id.to_string(), |i| i.t(id))
}

/// Translate a message ID with arguments.
pub fn t_args(
    world: &bevy::prelude::World,
    id: &str,
    args: &bevy_workbench::i18n::FluentArgs,
) -> String {
    world
        .get_resource::<I18n>()
        .map_or_else(|| id.to_string(), |i| i.t_args(id, args))
}
