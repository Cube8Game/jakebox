use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Game {
    pub name: String,
    pub description: String,
    pub script: String
}

#[derive(Serialize, Deserialize)]
struct Games {
    games: Vec<GameRegistry>
}

#[derive(Serialize, Deserialize)]
struct GameRegistry {
    path: String,
    name: String,
    description: String
}

impl Game {
    pub async fn get_all() -> Option<Vec<Self>> {
        let games: Games = serde_json::from_str(&reqwest::get(
            "https://raw.githubusercontent.com/Cube8Game/jakebox-games/refs/heads/main/games.json"
        ).await.ok()?.text().await.ok()?).ok()?;
        let mut result = Vec::new();
        for game_registry in games.games {
            result.push(Game {
                name: game_registry.name,
                description: game_registry.description,
                script: reqwest::get([
                    "https://raw.githubusercontent.com/Cube8Game/jakebox-games/refs/heads/main/games/", 
                    &game_registry.path
                ].concat()).await.ok()?.text().await.ok()?
            });
        }
        Some(result)
    }
}