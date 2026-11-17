//! JSON commands admitted at the persistent browser boundary.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub(super) enum BrowserCommand {
    PrepareStrategicScene {
        location: String,
        input_json: String,
        preparation: crate::presentation::generation::PreparationTicket,
    },
    SyncStrategicView {
        view: crate::strategic_scene::protocol::StrategicView,
    },
    RegionalMap {
        command: crate::presentation::regional_map::protocol::MapCommand,
    },
    ShowStrategicScene {
        scene: StrategicScene,
    },
    OrbitForge {
        delta_x: f32,
        delta_y: f32,
    },
    ZoomForge {
        delta: f32,
    },
    HideForgePreview,
    EnterTactical {
        server_addr: String,
        character_id: u64,
    },
    ExitTactical,
    /// The loading screen's Retry button.
    RetryTacticalLoad,
    Dialogue {
        request: crate::dialogue::BrowserDialogueRequest,
    },
    /// A click on the equipment slot HUD.
    PressSlotHud {
        press: crate::equipment::SlotHudPress,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub(super) enum StrategicScene {
    Forge {
        #[serde(default = "default_catalog_id")]
        catalog_id: String,
        design_json: Option<String>,
    },
}

fn default_catalog_id() -> String {
    "longsword".into()
}
