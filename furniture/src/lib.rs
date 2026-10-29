//! Furniture: chairs, tables and a test block (content in `assets/` and
//! `data/`), and sitting on chairs. A right click on a chair with an empty
//! hand seats the player on an invisible, still bat inside it; Shift stands
//! up and removes the seat. Sitting runs vanilla commands, so it needs cheats.
use horizon_mod_sdk::{Mod, Registration, command::run_quiet, export_mod, game, input};

const CHAIR: &str = "furniture:chair";
const SEAT_TAG: &str = "furniture_seat";
const REACH: f32 = 4.5;
/// A command line is cut at 256 characters (chat's limit), so seat NBT
/// stays short.
/// Below the block's floor, so the rider's feet sit on the seat (10/16 up).
const SEAT_Y: f64 = -0.8;

struct Furniture {
    use_down: bool,
    sneak_down: bool,
    seat: Option<[f64; 3]>,
}

fn remove_seats() {
    run_quiet(&[format!(
        "kill @e[type=minecraft:bat,tag={SEAT_TAG},distance=..4]"
    )]);
}

/// Seats whose chair is gone (broken, burnt, pushed): their rider falls off.
fn drop_orphan_seats() {
    run_quiet(&[format!(
        "execute as @e[type=minecraft:bat,tag={SEAT_TAG},distance=..16] at @s unless block ~ ~0.9 ~ {CHAIR} run kill @s"
    )]);
}

fn sit(x: i32, y: i32, z: i32, yaw: i32) -> [f64; 3] {
    let (sx, sy, sz) = (x as f64 + 0.5, y as f64 + SEAT_Y, z as f64 + 0.5);
    run_quiet(&[
        format!("kill @e[type=minecraft:bat,tag={SEAT_TAG},distance=..4]"),
        format!("rotate @s {yaw} 0"),
        format!(
            "summon bat {sx} {sy} {sz} {{NoAI:1b,NoGravity:1b,Silent:1b,Invulnerable:1b,PersistenceRequired:1b,attributes:[{{id:max_health,base:1}}],Tags:[{SEAT_TAG}],active_effects:[{{id:invisibility,duration:-1,show_particles:0b}}]}}"
        ),
        format!("ride @s mount @e[type=minecraft:bat,tag={SEAT_TAG},limit=1,sort=nearest]"),
    ]);
    [sx, sy, sz]
}

fn pressed(name: &str, was: &mut bool) -> bool {
    let down = input::mapping(name).is_some_and(|m| m.down);
    let edge = down && !*was;
    *was = down;
    edge
}

impl Mod for Furniture {
    fn init() -> Result<(Self, Registration), String> {
        Ok((
            Furniture {
                use_down: false,
                sneak_down: false,
                seat: None,
            },
            Registration::new().on_tick(),
        ))
    }

    fn tick(&mut self, tick: u64) {
        if tick % 10 == 0 {
            drop_orphan_seats();
        }
        let used = pressed("key.use", &mut self.use_down);
        let sneaked = pressed("key.sneak", &mut self.sneak_down);
        if self.seat.is_some() {
            if sneaked {
                remove_seats();
                self.seat = None;
            }
            return;
        }
        let empty_hand = input::held().is_none_or(|h| h.id == "minecraft:air");
        if !used || !empty_hand {
            return;
        }
        if let Some(hit) = game::raycast(REACH)
            && hit.block.split('[').next() == Some(CHAIR)
        {
            let yaw = match hit
                .block
                .split("facing=")
                .nth(1)
                .map(|f| f.trim_end_matches(']'))
            {
                Some("north") => 180,
                Some("east") => -90,
                Some("west") => 90,
                _ => 0,
            };
            self.seat = Some((sit(hit.block_x, hit.block_y, hit.block_z, yaw)));
        }
    }
}

export_mod!(Furniture);
