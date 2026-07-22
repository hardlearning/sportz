use std::sync::Arc;
use sea_orm::DatabaseConnection;


use crate::ws::connection::ConnectionManager;

pub type SharedState = Arc<AppState>;

pub struct AppState {
    pub connection: ConnectionManager,
    pub db: DatabaseConnection,
}

impl AppState {
    pub fn new(db: DatabaseConnection) -> Self {
        let connection = ConnectionManager::new();
        Self { connection, db }
    }
}