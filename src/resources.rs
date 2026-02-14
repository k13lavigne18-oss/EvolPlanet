use bevy::prelude::*;
use r2d2::Pool;
use r2d2_postgres::PostgresConnectionManager;
use postgres::NoTls;

#[derive(Resource)]
pub struct MoveTimer(pub Timer);

impl MoveTimer {
    pub fn new(timer: Timer) -> Self {
        Self(timer)
    }
}

#[derive(Resource)]
pub struct InputBuffer(pub Vec2);

#[derive(Resource)]
pub struct ChatLog {
    pub messages: Vec<(String, Timer)>,
}

#[derive(Resource)]
pub struct ChatMenuState {
    pub is_open: bool,
    pub selected_index: usize,
}

#[derive(Resource)]
pub struct EmojiConfig {
    pub s_key: String,
    pub d_key: String,
}

#[derive(Resource)]
pub struct EmojiSelectState {
    pub is_open: bool,
    pub target_key: Option<KeyCode>,
    pub selected_index: usize,
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    Login,
    Playing,
}

#[derive(PartialEq, Copy, Clone)]
pub enum AccountMode {
    Login,
    Create,
}

#[derive(Resource)]
pub struct AccountState {
    pub mode: AccountMode,
    pub username: String,
    pub password: String,
    pub is_typing_password: bool,
    pub error_msg: String,
}

#[derive(Resource)]
pub struct NotificationState {
    pub message: String,
    pub timer: Timer,
    pub is_visible: bool,
}

#[derive(Resource)]
pub struct CurrentUser {
    pub username: String,
    pub grid_x: i64,
    pub grid_y: i64,
    pub words: Vec<String>,
}

// Defaultの実装
impl Default for CurrentUser {
    fn default() -> Self {
        Self {
            username: "".to_string(),
            grid_x: 0,
            grid_y: 0,
            words: Vec::new(),
        }
    }
}

// 【新規】セリフデータを保持するリソース
#[derive(Resource, Default)]
pub struct BotDialogues {
    pub lines: Vec<String>,
}

pub const EMOJI_LIST: [&str; 20] = [
    "😁", "😭", "😡", "😇", "🤔", 
    "🤮", "💩", "👻", "💀", "👽",
    "👾", "🤖", "🔥", "💢", "💦",
    "💤", "❤️", "💔", "👀", "🧠"
];

#[derive(Resource, Clone)]
pub struct DbPool(pub Pool<PostgresConnectionManager<NoTls>>);