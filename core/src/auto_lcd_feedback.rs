//! Automatic LCD policy for routed AL80 input actions.
//!
//! The firmware action remains authoritative. This module only decides
//! whether host-side LCD feedback should be omitted, delegated to the
//! existing audio watcher, or rendered through the typed generic LCD path.
//!
//! V1 deliberately does not invent absolute RGB/brightness/speed values.

use crate::lcd_feedback::LcdFeedback;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoLcdPolicy {
    None,
    AudioWatcher,
    Feedback(LcdFeedback),
}

pub fn auto_lcd_policy(action: u8) -> Result<AutoLcdPolicy, String> {
    match action {
        0 => Ok(AutoLcdPolicy::None),

        // Volume/Mute must use the existing watcher so the LCD displays
        // the actual host volume percentage / mute state.
        1..=3 => Ok(AutoLcdPolicy::AudioWatcher),

        // These actions have no trustworthy absolute state available in
        // the event frame. Display the allowlisted action ID instead of
        // fabricating brightness, hue, speed, navigation, or media state.
        4..=20 | 23 => Ok(AutoLcdPolicy::Feedback(LcdFeedback::parse(
            "ACTION",
            &action.to_string(),
        )?)),

        // Resulting state is explicit for OFF/ON actions.
        21 => Ok(AutoLcdPolicy::Feedback(LcdFeedback::parse("SNAKE", "OFF")?)),

        22 => Ok(AutoLcdPolicy::Feedback(LcdFeedback::parse("SNAKE", "ON")?)),

        // The event explicitly means scene OFF.
        24 => Ok(AutoLcdPolicy::Feedback(LcdFeedback::parse("SCENE", "OFF")?)),

        _ => Err(format!("unsupported automatic LCD action: {action}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_allowlisted_actions_have_policy() {
        for action in 0..=24 {
            assert!(auto_lcd_policy(action).is_ok(), "action {action}");
        }

        assert!(auto_lcd_policy(25).is_err());
    }

    #[test]
    fn none_and_audio_actions_do_not_use_generic_frames() {
        assert_eq!(auto_lcd_policy(0).unwrap(), AutoLcdPolicy::None,);

        for action in 1..=3 {
            assert_eq!(
                auto_lcd_policy(action).unwrap(),
                AutoLcdPolicy::AudioWatcher,
            );
        }
    }

    #[test]
    fn rgb_actions_do_not_invent_absolute_values() {
        for action in 15..=20 {
            match auto_lcd_policy(action).unwrap() {
                AutoLcdPolicy::Feedback(feedback) => {
                    assert_eq!(feedback.kind_token(), "ACTION",);
                    assert_eq!(feedback.value_token(), action.to_string(),);
                }
                other => panic!("unexpected RGB policy: {other:?}"),
            }
        }
    }

    #[test]
    fn explicit_effect_states_use_typed_feedback() {
        let snake_off = match auto_lcd_policy(21).unwrap() {
            AutoLcdPolicy::Feedback(value) => value,
            other => panic!("unexpected: {other:?}"),
        };

        assert_eq!(snake_off.kind_token(), "SNAKE");
        assert_eq!(snake_off.value_token(), "OFF");

        let snake_on = match auto_lcd_policy(22).unwrap() {
            AutoLcdPolicy::Feedback(value) => value,
            other => panic!("unexpected: {other:?}"),
        };

        assert_eq!(snake_on.kind_token(), "SNAKE");
        assert_eq!(snake_on.value_token(), "ON");

        let toggle = match auto_lcd_policy(23).unwrap() {
            AutoLcdPolicy::Feedback(value) => value,
            other => panic!("unexpected: {other:?}"),
        };

        assert_eq!(toggle.kind_token(), "ACTION");
        assert_eq!(toggle.value_token(), "23");

        let scene = match auto_lcd_policy(24).unwrap() {
            AutoLcdPolicy::Feedback(value) => value,
            other => panic!("unexpected: {other:?}"),
        };

        assert_eq!(scene.kind_token(), "SCENE");
        assert_eq!(scene.value_token(), "OFF");
    }
}
