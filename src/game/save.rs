//! Persist / load hunt state.

use super::Persist;

pub(super) fn load_save() -> Option<Persist> {
    let raw = read_save()?;
    match parse_save(&raw) {
        Some(p) => Some(p),
        None => {
            // A save this build cannot read would only hang CONTINUE; drop it
            // so the title offers NEW HUNT alone.
            log::warn!("dropping unreadable save ({} bytes)", raw.len());
            clear_save();
            None
        }
    }
}

/// Parse a save blob; `None` if it is not a save this build understands.
pub(super) fn parse_save(raw: &str) -> Option<Persist> {
    serde_json::from_str(raw).ok()
}

pub(super) fn clear_save() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.remove_item("claymore.save.v1");
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = std::fs::remove_file(save_path());
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn save_path() -> std::path::PathBuf {
    #[cfg(test)]
    {
        thread_local! {
            static PATH: std::path::PathBuf = {
                let id = std::thread::current().id();
                std::env::temp_dir().join(format!("claymore-test-save-{id:?}.json"))
            };
        }
        PATH.with(|p| p.clone())
    }
    #[cfg(not(test))]
    {
        std::path::PathBuf::from("claymore.save.json")
    }
}

pub(super) fn read_save() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window()?;
        let storage = window.local_storage().ok()??;
        storage.get_item("claymore.save.v1").ok()?
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::fs::read_to_string(save_path()).ok()
    }
}

pub(super) fn write_save(s: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item("claymore.save.v1", s);
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = std::fs::write(save_path(), s);
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;
    use crate::combat::{current_unit, PlayerAction, Side};
    use crate::dialog::{SceneId, SceneState};

    // Saves written by main at a9841f1 (before #21/#22/#23/#24), produced by
    // that build's own `Persist` serializer.
    const OLD_WORLD: &str = include_str!("testdata/a9841f1-world.json");
    const OLD_DOGA: &str = include_str!("testdata/a9841f1-doga-combat.json");
    const OLD_PABURO_RAKI: &str = include_str!("testdata/a9841f1-paburo-raki-combat.json");
    // a9841f1 persisted `mode: Scene` after the Doga result but not the scene.
    const OLD_SCENE: &str = include_str!("testdata/a9841f1-scene-after-doga.json");

    fn continue_from(raw: &str) -> Game {
        write_save(raw);
        let mut game = Game::new();
        assert!(game.has_save, "old save must offer CONTINUE");
        game.continue_hunt();
        clear_save();
        game
    }

    #[test]
    fn old_saves_parse() {
        for raw in [OLD_WORLD, OLD_DOGA, OLD_PABURO_RAKI, OLD_SCENE] {
            let p = parse_save(raw).expect("a9841f1 save parses");
            assert!(p.scene.is_none() && p.pending_encounter.is_none());
        }
    }

    #[test]
    fn old_world_save_continues_on_island() {
        let game = continue_from(OLD_WORLD);
        assert_eq!(game.mode, Mode::World);
    }

    #[test]
    fn old_scene_save_lands_on_island_not_blank_backdrop() {
        let game = continue_from(OLD_SCENE);
        assert_eq!(game.mode, Mode::World, "no scene to show: resume on the island");
        assert!(game.scene.is_none());
    }

    #[test]
    fn old_combat_saves_resume_and_rounds_progress() {
        for raw in [OLD_DOGA, OLD_PABURO_RAKI] {
            let mut game = continue_from(raw);
            assert_eq!(game.mode, Mode::Combat);
            let start = game.combat.as_ref().unwrap().round;
            for _ in 0..4000 {
                let Some(c) = game.combat.as_ref() else { break };
                if c.over.is_some() || c.round >= start + 3 {
                    break;
                }
                if game.combat_input_open()
                    && current_unit(c).map(|u| u.side == Side::Player).unwrap_or(false)
                {
                    game.combat_act(PlayerAction::Wait);
                }
                game.tick(0.05);
            }
            let progressed = game
                .combat
                .as_ref()
                .map(|c| c.over.is_some() || c.round >= start + 3)
                .unwrap_or(true);
            assert!(progressed, "combat from an old save must keep taking turns");
        }
    }

    #[test]
    fn scene_survives_save_and_continue() {
        let mut game = Game::new();
        game.new_hunt();
        game.scene = Some(SceneState::new(SceneId::RakiJoin));
        game.mode = Mode::Scene;
        game.persist();
        let raw = read_save().unwrap();
        let game = continue_from(&raw);
        assert_eq!(game.mode, Mode::Scene);
        assert_eq!(game.scene.as_ref().map(|s| s.id), Some(SceneId::RakiJoin));
    }

    #[test]
    fn unreadable_save_is_dropped() {
        write_save("{\"v\":1,\"world\":42}");
        assert!(load_save().is_none());
        assert!(read_save().is_none(), "garbage save is removed");
        let mut game = Game::new();
        assert!(!game.has_save);
        write_save("not json");
        game.has_save = true;
        game.continue_hunt();
        assert_eq!(game.mode, Mode::Title);
        assert!(!game.has_save, "CONTINUE hides; NEW HUNT remains");
    }

    #[test]
    fn resume_mode_table() {
        assert_eq!(resume_mode(Mode::Scene, false, false), Mode::World);
        assert_eq!(resume_mode(Mode::Result, false, true), Mode::Scene);
        assert_eq!(resume_mode(Mode::Result, false, false), Mode::World);
        assert_eq!(resume_mode(Mode::Intro, false, false), Mode::World);
        assert_eq!(resume_mode(Mode::Town, false, false), Mode::Town);
        assert_eq!(resume_mode(Mode::World, true, false), Mode::Combat);
    }
}
