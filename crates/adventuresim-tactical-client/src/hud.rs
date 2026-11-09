//! The tactical HUD is DOM in the browser page, not Bevy UI. The page loads
//! `tactical-hud.js` (`crates/adventuresim-stdb-module/static/`) before the
//! client boots; it builds the elements and defines the `window.hud_*`
//! functions this module calls.
//!
//! On wasm each function here is one `#[wasm_bindgen]` call into the page.
//! Callers push only when a value changed (they keep the last pushed value in
//! a [`LastPushed`]), so an idle frame crosses nothing into JavaScript.
//! Structured views cross as JSON, which the owning module serializes.
//!
//! The native client has no DOM and therefore no HUD: every function is a
//! no-op there.

/// The value a caller last sent to the page, so it can skip unchanged pushes.
#[derive(Debug)]
pub(crate) struct LastPushed<T>(Option<T>);

impl<T> Default for LastPushed<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T: PartialEq> LastPushed<T> {
    /// Records `value` and returns it when it differs from the last push.
    pub(crate) fn update(&mut self, value: T) -> Option<&T> {
        if self.0.as_ref() == Some(&value) {
            return None;
        }
        Some(self.0.insert(value))
    }
}

/// The aim reticle and its muzzle-blocked warning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Reticle {
    Hidden,
    Aiming,
    MuzzleBlocked,
}

impl Reticle {
    #[cfg_attr(
        not(target_family = "wasm"),
        expect(dead_code, reason = "native has no DOM")
    )]
    fn token(self) -> &'static str {
        match self {
            Self::Hidden => "hidden",
            Self::Aiming => "aiming",
            Self::MuzzleBlocked => "blocked",
        }
    }
}

/// A short line of HUD text the page shows in a fixed place.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HudText {
    /// "Guard: Lowered", bottom centre above the controls.
    Guard,
    /// Seconds left before a committed attack lands, at the screen centre.
    AttackTimer,
}

impl HudText {
    #[cfg_attr(
        not(target_family = "wasm"),
        expect(dead_code, reason = "native has no DOM")
    )]
    fn id(self) -> &'static str {
        match self {
            Self::Guard => "guard",
            Self::AttackTimer => "attack-timer",
        }
    }
}

/// A JSON view the page renders as a whole.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HudView {
    /// Incapacitation wheels: the player's at the centre and enemies' over
    /// their heads (`ui::wheel`).
    Wheels,
    /// The QWERTY equipment slot grid while a hand grabs (`equipment`).
    Slots,
    /// The loading screen (`loading_screen::view`).
    Loading,
    /// The result of the player's last attack; the page fades it out.
    AttackResult,
    /// The tactical outcome banner; `null` hides it.
    Outcome,
}

impl HudView {
    #[cfg_attr(
        not(target_family = "wasm"),
        expect(dead_code, reason = "native has no DOM")
    )]
    fn id(self) -> &'static str {
        match self {
            Self::Wheels => "wheels",
            Self::Slots => "slots",
            Self::Loading => "loading",
            Self::AttackResult => "attack-result",
            Self::Outcome => "outcome",
        }
    }
}

#[cfg(target_family = "wasm")]
mod js {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = window)]
        pub fn hud_visible(on: bool);
        #[wasm_bindgen(js_namespace = window)]
        pub fn hud_reticle(state: &str);
        #[wasm_bindgen(js_namespace = window)]
        pub fn hud_text(id: &str, text: &str);
        #[wasm_bindgen(js_namespace = window)]
        pub fn hud_view(id: &str, json: &str);
        /// Limb condition in percent: head, chest, stomach, left arm, right
        /// arm, left leg, right leg.
        #[wasm_bindgen(js_namespace = window)]
        pub fn hud_limbs(percent: &[f32]);
        /// An icon image the slot HUD refers to by `key`, as straight RGBA8.
        #[wasm_bindgen(js_namespace = window)]
        pub fn hud_icon(key: &str, width: u32, height: u32, rgba: &[u8]);
    }
}

/// Shows the tactical HUD, or hides it while the canvas shows strategic
/// scenes.
#[cfg(target_family = "wasm")]
pub(crate) fn visible(on: bool) {
    js::hud_visible(on);
}

pub(crate) fn reticle(state: Reticle) {
    #[cfg(target_family = "wasm")]
    js::hud_reticle(state.token());
    #[cfg(not(target_family = "wasm"))]
    let _ = state;
}

/// Sets a text line; empty hides it.
pub(crate) fn text(line: HudText, text: &str) {
    #[cfg(target_family = "wasm")]
    js::hud_text(line.id(), text);
    #[cfg(not(target_family = "wasm"))]
    let _ = (line, text);
}

pub(crate) fn view(view: HudView, json: &str) {
    #[cfg(target_family = "wasm")]
    js::hud_view(view.id(), json);
    #[cfg(not(target_family = "wasm"))]
    let _ = (view, json);
}

pub(crate) fn limbs(percent: &[f32; 7]) {
    #[cfg(target_family = "wasm")]
    js::hud_limbs(percent);
    #[cfg(not(target_family = "wasm"))]
    let _ = percent;
}

pub(crate) fn icon(key: &str, width: u32, height: u32, rgba: &[u8]) {
    #[cfg(target_family = "wasm")]
    js::hud_icon(key, width, height, rgba);
    #[cfg(not(target_family = "wasm"))]
    let _ = (key, width, height, rgba);
}

#[cfg(test)]
mod tests {
    use super::LastPushed;

    #[test]
    fn last_pushed_reports_only_changes() {
        let mut last = LastPushed::default();
        assert_eq!(last.update(1), Some(&1));
        assert_eq!(last.update(1), None);
        assert_eq!(last.update(2), Some(&2));
    }
}
