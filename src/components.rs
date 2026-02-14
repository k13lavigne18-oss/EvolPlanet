use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub struct GridPosition {
    pub x: i64,
    pub y: i64,
}

#[derive(Component)]
pub struct Vocabulary {
    pub words: Vec<String>,
}

#[derive(Component)]
pub struct PositionText;

#[derive(Component)]
pub struct DirectionText;

#[derive(Component)]
pub struct ChatDisplay;

#[derive(Component)]
pub struct ChatMenuDisplay;

#[derive(Component)]
pub struct PlayerEmoji;

#[derive(Component)]
pub struct EmojiTimer(pub Timer);

#[derive(Component)]
pub struct EmojiSelectMenuDisplay;

#[derive(Component)]
pub struct NotificationDisplay;

#[derive(Component)]
pub struct NotificationText;

#[derive(Component)]
pub struct SaveButton;

#[derive(Component)]
pub struct GameEntity; 

// --- ボット関連 ---

#[derive(Component)]
pub struct Bot;

#[derive(Component)]
pub struct BotSpawnPoint {
    pub x: i64,
    pub y: i64,
}

#[derive(Component)]
pub struct BotMoveTimer(pub Timer);

#[derive(Component)]
pub struct BotChatText;

#[derive(Component)]
pub struct BotChatTimer(pub Timer);

#[derive(Component)]
pub struct VoiceEffect;

// 【新規】ボットが話している間の停止タイマー
#[derive(Component)]
pub struct BotTalking(pub Timer);

// 【新規】そのボット固有のセリフ（生成時に決定・固定）
#[derive(Component)]
pub struct BotDialogueText(pub String);