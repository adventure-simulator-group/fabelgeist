//! What the player sets about the world: the scene, how fast years pass,
//! the climate, and what the plants may do.

use fabelgeist_botany::Biome;
use fabelgeist_botany::scenes::Scene;
use fabelgeist_botany::viewer::sim::Command;

/// How fast years pass while the world plays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pace {
    Slow,
    Normal,
    Fast,
    /// As fast as the simulation goes.
    Fastest,
}

impl Pace {
    pub const ALL: [Pace; 4] = [Pace::Slow, Pace::Normal, Pace::Fast, Pace::Fastest];

    /// Simulated years a wall-clock second.
    pub fn years_per_second(self) -> f32 {
        match self {
            Pace::Slow => 5.0,
            Pace::Normal => 20.0,
            Pace::Fast => 80.0,
            Pace::Fastest => f32::INFINITY,
        }
    }

    /// Play arrows its button shows.
    pub fn arrows(self) -> usize {
        match self {
            Pace::Slow => 1,
            Pace::Normal => 2,
            Pace::Fast => 3,
            Pace::Fastest => 4,
        }
    }

    pub fn describe(self) -> &'static str {
        match self {
            Pace::Slow => "5 years a second",
            Pace::Normal => "20 years a second",
            Pace::Fast => "80 years a second",
            Pace::Fastest => "As fast as it grows",
        }
    }
}

/// What the viewer asks of the simulation beyond plain commands.
pub enum Request {
    Send(Command),
    /// Start `Scene` afresh.
    Load(Scene),
}

pub struct World {
    pub scene: Scene,
    pub playing: bool,
    pub pace: Pace,
    /// At the valley floor, degrees Celsius.
    pub temperature: f32,
    /// A year's, millimetres.
    pub precipitation: f32,
    /// Whether plants shade the plants under them.
    pub canopy_shading: bool,
    /// Whether plants spread seed, and whether plants and vines die; kept
    /// from scene to scene.
    pub seeding: bool,
    pub mortality: bool,
}

impl World {
    pub fn new(scene: Scene) -> Self {
        let mut world = Self {
            scene,
            playing: true,
            pace: Pace::Normal,
            temperature: 0.0,
            precipitation: 0.0,
            canopy_shading: true,
            seeding: true,
            mortality: true,
        };
        world.adopt_climate(scene);
        world
    }

    /// The climate settings start from a scene's own climate.
    pub fn adopt_climate(&mut self, scene: Scene) {
        let climate = scene.build(1).climate;
        self.scene = scene;
        self.temperature = climate.temperature;
        self.precipitation = climate.precipitation;
    }

    /// What a new scene is told so it follows the settings.
    pub fn start(&self) -> [Command; 3] {
        [
            Command::Play(self.playing),
            Command::Seeding(self.seeding),
            Command::Mortality(self.mortality),
        ]
    }

    pub fn toggle_play(&mut self) -> Command {
        self.playing = !self.playing;
        Command::Play(self.playing)
    }

    /// Play at `pace`.
    pub fn play_at(&mut self, pace: Pace) -> [Command; 2] {
        self.playing = true;
        self.pace = pace;
        [Command::Speed(pace.years_per_second()), Command::Play(true)]
    }

    pub fn climate(&self) -> Command {
        Command::Climate {
            temperature: self.temperature,
            precipitation: self.precipitation,
        }
    }

    /// The biome the climate settings make.
    pub fn biome(&self) -> Biome {
        Biome::classify(self.temperature, self.precipitation)
    }
}

/// A scene's one-line pitch.
pub fn pitch(scene: Scene) -> &'static str {
    match scene {
        Scene::MixedForest => "Conifers, broadleaf trees and shrubs on level ground",
        Scene::BorealForest => "Cold hills of conifers over a shrub understory",
        Scene::Savanna => "Scattered baobabs over grass and shrubs",
        Scene::Rainforest => "Palms under a tall, warm, wet canopy",
        Scene::Castle => "Trees round the towers, ivy up the walls",
        Scene::EmptyCastle => "Bare ground round the castle, ready to sow",
    }
}

pub fn biome_name(biome: Biome) -> &'static str {
    match biome {
        Biome::Tundra => "Tundra",
        Biome::BorealForest => "Boreal forest",
        Biome::TemperateGrassland => "Temperate grassland",
        Biome::Shrubland => "Shrubland",
        Biome::TemperateSeasonalForest => "Temperate seasonal forest",
        Biome::TemperateRainforest => "Temperate rainforest",
        Biome::Desert => "Desert",
        Biome::Savanna => "Savanna",
        Biome::TropicalSeasonalForest => "Tropical seasonal forest",
        Biome::TropicalRainforest => "Tropical rainforest",
    }
}
