use reqwest::{Response, StatusCode};

use super::common::Connection;
use crate::common::{connection::{GameAction, GameRequest}, state::{Event, GameState}};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Reqwest Error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Error with status code {0}")]
    ServerError(StatusCode)
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct HttpConnection {
    client: reqwest::Client,
    pub server_ip: String, // TODO: Add some validation for the server IP.
    pub game_id: String,
    pub username: String
}

impl HttpConnection {
    const SERVER_LIST: [&'static str; 2] = [
        "https://box.jakewhibley.org",
        "https://jakebox.tunnl.gg", // Backup server
    ];

    pub fn new(server_ip: String, game_id: String, username: String) -> Self {
        Self { client: reqwest::Client::new(), server_ip, game_id, username }
    }

    pub async fn find_server() -> Option<String> {
        for server in Self::SERVER_LIST {
            if let Ok(response) = reqwest::get([server, "/api/unstable/status"].concat()).await {
                if response.status() == StatusCode::OK {
                    return Some(server.to_string());
                }
            }
        }
        None
    }
}

impl HttpConnection {
    fn url(&self, request_type: &str) -> String {
        [&self.server_ip, "/api/unstable/", request_type].concat()
    }

    fn game_url(&self, request_type: &str) -> String {
        self.url(&["games/", &self.game_id, "/", request_type].concat())
    }

    fn request(&self, action: Option<GameAction>) -> GameRequest {
        GameRequest::new(self.username.clone(), action)
    }
}

impl Connection<Response, Error> for HttpConnection {
    async fn host(server_ip: String, username: String) -> Result<Self>
        where Self: Sized
    {
        let mut connection = Self::new(server_ip, String::new(), username);
        let response = connection
            .client
            .post(connection.url("games"))
            .json(&connection.request(None))
            .send()
            .await
            .map_err(Error::ReqwestError)?;
        if response.status().is_success() {
            connection.game_id = response.text().await?;
            Ok(connection)
        } else {
            Err(Error::ServerError(response.status()))
        }
    }

    async fn join(server_ip: String, game_id: String, username: String) -> Result<Self>
        where Self: Sized
    {
        let connection = Self::new(server_ip, game_id, username);
        let response = connection
            .client
            .post(connection.game_url("join"))
            .json(&connection.request(None))
            .send()
            .await
            .map_err(Error::ReqwestError)?;
        if response.status().is_success() {
            Ok(connection)
        } else {
            Err(Error::ServerError(response.status()))
        }
    }

    async fn get_event(&self) -> Result<Event> {
        let response = self
            .client
            .post(self.game_url("event"))
            .json(&self.request(None))
            .send()
            .await
            .map_err(Error::ReqwestError)?;
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(Error::ServerError(response.status()))
        }
    }

    async fn get_state(&self) -> Result<GameState> {
        let response = self
            .client
            .get(self.game_url("state"))
            .json(&self.request(None))
            .send()
            .await
            .map_err(Error::ReqwestError)?;
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(Error::ServerError(response.status()))
        }
    }
    
    async fn send_action(&self, action: GameAction) -> Result<Response> {
        let response = self
            .client
            .post(self.game_url("action"))
            .json(&self.request(Some(action)))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(Error::ServerError(response.status()));
        }
        Ok(response)
    }
}