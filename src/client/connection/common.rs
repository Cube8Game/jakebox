use crate::common::{connection::GameAction, state::{Event, GameState}};

pub trait Connection<Response, RequestError> {
    fn host(server_ip: String, username: String) -> impl Future<Output = Result<Self, RequestError>> + Send
        where Self: Sized;
    fn join(server_ip: String, game_id: String, username: String) -> impl Future<Output = Result<Self, RequestError>> + Send
        where Self: Sized;
    fn get_event(&self) -> impl Future<Output = Result<Event, RequestError>> + Send;
    fn get_state(&self) -> impl Future<Output = Result<GameState, RequestError>> + Send;
    fn send_action(&self, action: GameAction) -> impl Future<Output = Result<Response, RequestError>> + Send;
}
