use super::{
    AnimPhase, BTN_PRESSED_COLOR, ControllerDirections, FALLBACK_BTN_COLOR, MultitouchPressed,
    TouchAction, TouchAnimFrames, TouchAnimState, TouchControllerOverlay, TouchControllerZone,
    TouchNormalImage, TouchOverlayEnabled, TouchOverlayRoot, TouchPressedImage,
};
use bevy::input::touch::Touches;
use bevy::prelude::*;

/// Update button visuals for both two-texture and animated-frame buttons.
/// Checks both single-pointer (Interaction) and multitouch (MultitouchPressed).
pub fn update_touch_button_visuals(
    touches: Res<Touches>,
    multitouch: Res<MultitouchPressed>,
    mut two_texture_buttons: Query<
        (
            &Interaction,
            &TouchAction,
            &mut BackgroundColor,
            &TouchNormalImage,
            &TouchPressedImage,
            Option<&mut ImageNode>,
        ),
        (Without<TouchAnimFrames>, Without<TouchControllerZone>),
    >,
    mut anim_buttons: Query<(
        &Interaction,
        &TouchAction,
        &mut TouchAnimState,
        &TouchAnimFrames,
        &mut ImageNode,
    )>,
) {
    let has_touches = touches.iter().next().is_some();

    for (interaction, action, mut bg, normal, pressed_img, image_node) in
        two_texture_buttons.iter_mut()
    {
        let is_pressed = multitouch.0.contains(&action.0)
            || (!has_touches && *interaction == Interaction::Pressed);
        if is_pressed {
            if let Some(mut img) = image_node
                && let Some(ref handle) = pressed_img.0
            {
                img.image = handle.clone();
            }
            *bg = BackgroundColor(BTN_PRESSED_COLOR);
        } else {
            if let Some(mut img) = image_node
                && let Some(ref handle) = normal.0
            {
                img.image = handle.clone();
            }
            *bg = BackgroundColor(FALLBACK_BTN_COLOR);
        }
    }

    for (interaction, action, mut anim, frames, mut img) in anim_buttons.iter_mut() {
        let is_pressed = multitouch.0.contains(&action.0)
            || (!has_touches && *interaction == Interaction::Pressed);
        let should_start_press =
            is_pressed && anim.phase != AnimPhase::Pressing && anim.phase != AnimPhase::Held;
        let should_start_release =
            !is_pressed && (anim.phase == AnimPhase::Pressing || anim.phase == AnimPhase::Held);

        if should_start_press {
            anim.phase = AnimPhase::Pressing;
            anim.current_frame = 1;
            anim.timer.reset();
            if let Some(handle) = frames.0.get(1) {
                img.image = handle.clone();
            }
        } else if should_start_release {
            anim.phase = AnimPhase::Releasing;
            anim.current_frame = 3;
            anim.timer.reset();
            if let Some(handle) = frames.0.get(3) {
                img.image = handle.clone();
            }
        }
    }
}

/// Tick animation timers and advance frames.
pub fn tick_touch_button_animations(
    time: Res<Time>,
    mut buttons: Query<(&mut TouchAnimState, &TouchAnimFrames, &mut ImageNode)>,
) {
    for (mut anim, frames, mut img) in buttons.iter_mut() {
        anim.timer.tick(time.delta());
        if !anim.timer.is_finished() {
            continue;
        }
        match anim.phase {
            AnimPhase::Pressing => {
                anim.phase = AnimPhase::Held;
                anim.current_frame = 2;
                if let Some(handle) = frames.0.get(2) {
                    img.image = handle.clone();
                }
            }
            AnimPhase::Releasing => {
                anim.phase = AnimPhase::Idle;
                anim.current_frame = 0;
                if let Some(handle) = frames.0.first() {
                    img.image = handle.clone();
                }
            }
            _ => {}
        }
    }
}

/// Update controller direction overlays based on active controller directions.
pub fn update_controller_overlays(
    dirs: Res<ControllerDirections>,
    mut overlays: Query<(&TouchControllerOverlay, &mut Visibility)>,
) {
    for (overlay, mut vis) in overlays.iter_mut() {
        *vis = if dirs.0.contains(&overlay.0) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// Toggle touch overlay visibility.
pub fn toggle_touch_overlay(
    mut enabled: ResMut<TouchOverlayEnabled>,
    mut overlays: Query<&mut Visibility, With<TouchOverlayRoot>>,
) {
    enabled.0 = !enabled.0;
    let vis = if enabled.0 {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut v in overlays.iter_mut() {
        *v = vis;
    }
}
