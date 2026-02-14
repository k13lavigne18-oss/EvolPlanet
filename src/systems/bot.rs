use bevy::prelude::*;
use crate::constants::TILE_SIZE;
use crate::components::*;
use crate::resources::BotDialogues; // セリフリソースを使う
use crate::map::{is_bot_spawn, is_obstacle};
use rand::Rng;
use rand::seq::SliceRandom; // ランダム選択用

pub fn spawn_visible_bots(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    camera_query: Query<(&GlobalTransform, &OrthographicProjection), With<Camera>>,
    window_query: Query<&Window>,
    existing_bots: Query<&BotSpawnPoint, With<Bot>>,
    // 【新規】セリフデータを受け取る
    bot_dialogues: Res<BotDialogues>,
) {
    let Ok((cam_transform, projection)) = camera_query.get_single() else { return };
    let Ok(window) = window_query.get_single() else { return };

    let jp_font = asset_server.load("fonts/NotoSansJP-Bold.ttf");

    let view_half_width = window.resolution.width() / 2.0 * projection.scale;
    let view_half_height = window.resolution.height() / 2.0 * projection.scale;
    let cam_pos = cam_transform.translation();

    let buffer = 2.0; 
    let start_x = ((cam_pos.x - view_half_width) / TILE_SIZE - buffer).floor() as i64;
    let end_x = ((cam_pos.x + view_half_width) / TILE_SIZE + buffer).ceil() as i64;
    let start_y = ((cam_pos.y - view_half_height) / TILE_SIZE - buffer).floor() as i64;
    let end_y = ((cam_pos.y + view_half_height) / TILE_SIZE + buffer).ceil() as i64;

    let existing_positions: Vec<(i64, i64)> = existing_bots.iter()
        .map(|p| (p.x, p.y))
        .collect();

    for x in start_x..=end_x {
        for y in start_y..=end_y {
            if is_bot_spawn(x, y) && !existing_positions.contains(&(x, y)) {
                // セリフをここで決定して渡す
                let dialogue = if let Some(line) = bot_dialogues.lines.choose(&mut rand::thread_rng()) {
                    line.clone()
                } else {
                    "...".to_string()
                };

                spawn_single_bot(&mut commands, x, y, &jp_font, dialogue);
            }
        }
    }
}

// 引数に dialogue を追加
fn spawn_single_bot(commands: &mut Commands, x: i64, y: i64, font: &Handle<Font>, dialogue: String) {
    let mut rng = rand::thread_rng();
    let color = Color::srgb(0.0, 0.0, 1.0);

    commands.spawn((
        Bot,
        GameEntity,
        BotSpawnPoint { x, y },
        BotMoveTimer(Timer::from_seconds(rng.gen_range(1.0..3.0), TimerMode::Repeating)),
        
        // 生成時に決まったセリフを持たせる
        BotDialogueText(dialogue),
        // 会話中タイマー
        BotTalking(Timer::from_seconds(0.0, TimerMode::Once)), 
        
        Transform::from_xyz(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE, 0.5), 
        Sprite {
            color,
            custom_size: Some(Vec2::new(TILE_SIZE * 0.8, TILE_SIZE * 0.8)),
            ..default()
        },
    ))
    .with_children(|parent| {
        parent.spawn((
            Text2d::new(""),
            TextFont { font: font.clone(), font_size: 20.0, ..default() },
            TextColor(Color::BLACK),
            TextLayout::new(JustifyText::Center, LineBreak::NoWrap),
            Transform::from_xyz(0.0, TILE_SIZE * 0.8, 10.0),
            BotChatText,
            BotChatTimer(Timer::from_seconds(3.0, TimerMode::Once)),
        ));
    });
}

pub fn despawn_far_bots(
    mut commands: Commands,
    camera_query: Query<(&GlobalTransform, &OrthographicProjection), With<Camera>>,
    window_query: Query<&Window>,
    bot_query: Query<(Entity, &Transform), With<Bot>>,
) {
    let Ok((cam_transform, projection)) = camera_query.get_single() else { return };
    let Ok(window) = window_query.get_single() else { return };

    let cam_pos = cam_transform.translation();
    let limit_dist = window.resolution.width().max(window.resolution.height()) * projection.scale * 2.0;

    for (entity, transform) in &bot_query {
        if transform.translation.distance(cam_pos) > limit_dist {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn bot_wander_system(
    time: Res<Time>,
    // BotTalking も取得
    mut bot_query: Query<(&mut Transform, &BotSpawnPoint, &mut BotMoveTimer, &mut BotTalking), With<Bot>>,
) {
    let mut rng = rand::thread_rng();

    for (mut transform, spawn_point, mut timer, mut talking) in &mut bot_query {
        // 会話タイマー進行
        talking.0.tick(time.delta());

        // 話している最中は移動処理をスキップ
        if !talking.0.finished() {
            continue;
        }

        timer.0.tick(time.delta());

        if timer.0.finished() {
            let direction = rng.gen_range(0..4);
            let (dx, dy) = match direction {
                0 => (0, 1),
                1 => (0, -1),
                2 => (-1, 0),
                3 => (1, 0),
                _ => (0, 0),
            };

            let current_grid_x = (transform.translation.x / TILE_SIZE).round() as i64;
            let current_grid_y = (transform.translation.y / TILE_SIZE).round() as i64;

            let next_x = current_grid_x + dx;
            let next_y = current_grid_y + dy;

            if (next_x - spawn_point.x).abs() <= 5 && (next_y - spawn_point.y).abs() <= 5 {
                if !is_obstacle(next_x, next_y) {
                    transform.translation.x = next_x as f32 * TILE_SIZE;
                    transform.translation.y = next_y as f32 * TILE_SIZE;
                }
            }
            
            timer.0.set_duration(std::time::Duration::from_secs_f32(rng.gen_range(0.5..2.0)));
            timer.0.reset();
        }
    }
}

pub fn update_bot_chat(
    time: Res<Time>,
    mut query: Query<(&mut Text2d, &mut BotChatTimer), With<BotChatText>>,
) {
    for (mut text, mut timer) in &mut query {
        if !text.0.is_empty() {
            timer.0.tick(time.delta());
            if timer.0.finished() {
                text.0 = "".to_string();
            }
        }
    }
}