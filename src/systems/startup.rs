use bevy::prelude::*;
use crate::constants::*;
use crate::components::*;
use crate::resources::*;
use crate::database;
use r2d2_postgres::PostgresConnectionManager;
use postgres::NoTls;
use std::fs;
use std::env; // 環境変数読み込み用

pub fn setup(
    mut commands: Commands,
) {
    // 【修正】環境変数 DB_HOST があれば使い、なければ localhost を使う
    // Docker内では "db"、ローカル開発では "localhost" になります
    let db_host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
    
    // 接続文字列を動的に生成
    let db_url = format!("host={} user=postgres password=password dbname=postgres", db_host);

    println!("Connecting to DB at: {}", db_host);

    let manager = PostgresConnectionManager::new(
        db_url.parse().unwrap(),
        NoTls,
    );
    
    // 接続待機リトライロジックがないため、コンテナ起動順によっては即死する可能性がありますが
    // restart: alwaysを入れているので再起動してつながります。
    let pool = r2d2::Pool::new(manager).expect("Failed to create DB pool.");
    
    if let Err(e) = database::init_db(&pool) {
        eprintln!("DB Init Error: {}", e);
    }
    
    commands.insert_resource(DbPool(pool));
    commands.spawn(Camera2d);

    // セリフ読み込み
    let mut dialogues = BotDialogues::default();
    // Docker内パスに対応するため、相対パスはそのまま使用（WORKDIRが/appなのでOK）
    match fs::read_to_string("assets/bot_dialogues.txt") {
        Ok(content) => {
            dialogues.lines = content.lines().map(|s| s.to_string()).collect();
            println!("Loaded {} lines of dialogue.", dialogues.lines.len());
        }
        Err(e) => {
            eprintln!("Failed to load bot dialogues: {}", e);
            dialogues.lines.push("No dialogues found.".to_string());
        }
    }
    commands.insert_resource(dialogues);
}

pub fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    current_user: Res<CurrentUser>,
) {
    let jp_font = asset_server.load("fonts/NotoSansJP-Bold.ttf");
    let emoji_font = asset_server.load("fonts/NotoEmoji-Bold.ttf");

    commands.spawn((
        Player,
        Transform::from_xyz(0.0, 0.0, 0.0), 
        GridPosition { 
            x: current_user.grid_x, 
            y: current_user.grid_y 
        },
        Vocabulary {
            words: current_user.words.clone(),
        },
        GameEntity,
    ))
    .with_children(|parent| {
        let radius = 4;
        parent.spawn((
            Transform::from_xyz(0.0, 0.0, -1.0), 
            Visibility::Hidden, 
            VoiceEffect,
        ))
        .with_children(|voice_parent| {
            for dx in -radius..=radius {
                for dy in -radius..=radius {
                    voice_parent.spawn((
                        Sprite {
                            color: Color::srgba(0.0, 1.0, 1.0, 0.3), 
                            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                            ..default()
                        },
                        Transform::from_xyz(dx as f32 * TILE_SIZE, dy as f32 * TILE_SIZE, 0.0),
                    ));
                }
            }
        });
    });

    commands.spawn(Node {
        width: Val::Px(TILE_SIZE * 0.9),
        height: Val::Px(TILE_SIZE * 0.9),
        position_type: PositionType::Absolute,
        left: Val::Percent(50.0),
        top: Val::Percent(50.0),
        margin: UiRect {
            left: Val::Px(-(TILE_SIZE * 0.9) / 2.0),
            top: Val::Px(-(TILE_SIZE * 0.9) / 2.0),
            ..default()
        },
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    })
    .insert(BackgroundColor(PLAYER_COLOR))
    .insert(GameEntity)
    .with_children(|parent| {
        parent.spawn((
            Text::new(""), 
            TextFont { font: emoji_font.clone(), font_size: 30.0, ..default() },
            TextColor(Color::WHITE), 
            TextLayout::new(JustifyText::Center, LineBreak::NoWrap), 
            PlayerEmoji, 
            EmojiTimer(Timer::from_seconds(3.0, TimerMode::Once)),
        ));
    });

    commands.spawn((
        Text::new("(0, 0)"),
        TextFont { font: jp_font.clone(), font_size: 24.0, ..default() },
        TextColor(Color::BLACK),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            left: Val::Px(20.0),
            ..default()
        },
        PositionText,
        GameEntity,
    ));

    commands.spawn((
        Text::new("."),
        TextFont { font: jp_font.clone(), font_size: 40.0, ..default() },
        TextColor(Color::BLACK),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            right: Val::Px(30.0),
            ..default()
        },
        DirectionText,
        GameEntity,
    ));

    commands.spawn((
        Text::new(""),
        TextFont { font: jp_font.clone(), font_size: 18.0, ..default() },
        TextColor(Color::BLACK),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(20.0),
            ..default()
        },
        ChatDisplay,
        GameEntity,
    ));

    commands.spawn((
        Text::new(""),
        TextFont { font: jp_font.clone(), font_size: 24.0, ..default() },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(120.0),
            left: Val::Px(20.0),
            display: Display::None,
            padding: UiRect::all(Val::Px(15.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        ChatMenuDisplay,
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
        BorderColor(Color::WHITE),
        BorderRadius::all(Val::Px(10.0)),
        GameEntity,
    ));

    commands.spawn((
        Text::new(""),
        TextFont { font: emoji_font.clone(), font_size: 30.0, ..default() },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(20.0),
            right: Val::Px(50.0),
            display: Display::None,
            padding: UiRect::all(Val::Px(20.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        EmojiSelectMenuDisplay,
        BackgroundColor(Color::srgba(0.0, 0.0, 0.5, 0.9)),
        BorderColor(Color::WHITE),
        BorderRadius::all(Val::Px(10.0)),
        GameEntity,
    ));

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            right: Val::Px(20.0),
            display: Display::None,
            padding: UiRect::all(Val::Px(15.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.2, 0.8, 0.2, 0.9)),
        BorderRadius::all(Val::Px(10.0)),
        NotificationDisplay,
        ZIndex(200),
        GameEntity,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new(""),
            TextFont { font: jp_font.clone(), font_size: 20.0, ..default() },
            TextColor(Color::WHITE),
            NotificationText,
        ));
    });

    commands.spawn((
        Button,
        Node {
            width: Val::Px(80.0),
            height: Val::Px(40.0),
            position_type: PositionType::Absolute,
            bottom: Val::Px(60.0),
            left: Val::Px(20.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.2, 0.2, 0.8, 1.0)),
        BorderRadius::all(Val::Px(5.0)),
        crate::components::SaveButton,
        GameEntity,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("SAVE"),
            TextFont { font: jp_font.clone(), font_size: 20.0, ..default() },
            TextColor(Color::WHITE),
        ));
    });
}