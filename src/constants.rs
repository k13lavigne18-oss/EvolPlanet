use bevy::prelude::Color;

// 元の設定を維持（見た目が変わらないように）
pub const TILE_SIZE: f32 = 40.0;
pub const GRID_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
pub const PLAYER_COLOR: Color = Color::BLACK;

// 以前のリクエストに合わせて1兆に更新
pub const FIELD_LIMIT: i64 = 1_000_000_000_000;

// 【新規追加】プレイヤーの移動間隔 (秒)
// ここが「聖域」として切り出された設定値です
pub const PLAYER_MOVE_INTERVAL: f32 = 1.0;