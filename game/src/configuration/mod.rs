use std::collections::HashSet;
use std::fs::File;

#[derive(Debug, serde::Deserialize)]
pub enum Camera {
    Follow,
    Freelook {
        position: [f32; 3],
        direction: [f32; 3],
    },
}

#[derive(Debug, serde::Deserialize)]
pub struct Model {
    pub name: String,
    pub location: String,
}

#[derive(Debug, serde::Deserialize)]
pub enum Entity {
    Player {
        model_name: String,
        start_position: [f32; 3],
        max_velocity: f32,
    },
    Static {
        model_name: String,
        start_position: [f32; 3],
    },
}

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub models: Vec<Model>,
    pub entities: Vec<Entity>,
    pub cameras: Vec<Camera>,
}

impl Config {
    pub fn default() -> Self {
        Self {
            models: vec![],
            entities: vec![],
            cameras: vec![],
        }
    }
    fn is_valid(&self) -> bool {
        let mut uniq = HashSet::new();
        let model_names_uniq = self.models.iter().all(|x| uniq.insert(x.name.clone()));
        model_names_uniq
    }
    pub fn load_config(path: &str) -> Self {
        match File::open(&path) {
            Ok(f) => {
                let config: Result<Config, ron::Error> = ron::de::from_reader(f);
                match config {
                    Ok(config) => {
                        if config.is_valid() {
                            config
                        } else {
                            Config::default()
                        }
                    }
                    Err(_) => Self::default(),
                }
            }
            Err(_) => Self::default(),
        }
    }
}
