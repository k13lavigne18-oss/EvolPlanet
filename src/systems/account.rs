use bevy::prelude::*;
use bevy::input::keyboard::{KeyboardInput, Key};
use crate::resources::*;
// use crate::components::*; // Playerなどはここでは触らないので削除OK
use crate::database; 
use std::fs::File;
use std::io::Write;

#[derive(Component)]
pub struct AccountMenuDisplay;
#[derive(Component)]
pub struct AccountInputText;

pub fn setup_account_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let jp_font = asset_server.load("fonts/NotoSansJP-Bold.ttf");

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.9, 0.9, 0.9, 1.0)),
        AccountMenuDisplay,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new(""),
            TextFont { font: jp_font.clone(), font_size: 30.0, ..default() },
            TextColor(Color::BLACK),
            AccountInputText,
            TextLayout::new(JustifyText::Center, LineBreak::NoWrap),
        ));

        parent.spawn((
            Text::new("Note: For performance efficiency,\nID & Password cannot be changed or recovered.\nPlease keep them safe."),
            TextFont { font: jp_font.clone(), font_size: 16.0, ..default() },
            TextColor(Color::srgba(0.8, 0.2, 0.2, 1.0)), 
            Node {
                margin: UiRect::top(Val::Px(30.0)),
                ..default()
            },
            TextLayout::new(JustifyText::Center, LineBreak::NoWrap),
        ));
    });
}

pub fn handle_account_input(
    mut keyboard_events: EventReader<KeyboardInput>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<AccountState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut text_query: Query<&mut Text, With<AccountInputText>>,
    
    mut notification: ResMut<NotificationState>,
    db_pool: Res<DbPool>,
    
    // データを反映するためのリソース
    mut current_user: ResMut<CurrentUser>,
    // mut player_query: Query<...> ← これは削除（まだプレイヤーがいないため）
    mut emoji_config: ResMut<EmojiConfig>,
) {
    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        state.mode = AccountMode::Login;
        state.error_msg = "".to_string();
    }
    if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        state.mode = AccountMode::Create;
        state.error_msg = "".to_string();
    }

    if keyboard_input.just_pressed(KeyCode::Tab) {
        state.is_typing_password = !state.is_typing_password;
    }

    for event in keyboard_events.read() {
        if !event.state.is_pressed() { continue; }
        if let Key::Character(ref sm) = event.logical_key {
            let c_str = sm.as_str();
            if c_str.len() == 1 {
                let c = c_str.chars().next().unwrap();
                if c.is_ascii_alphanumeric() {
                    if !state.is_typing_password {
                        if state.username.len() < 12 { state.username.push(c); }
                    } else {
                        if state.password.len() < 12 { state.password.push(c); }
                    }
                }
            }
        }
    }

    if keyboard_input.just_pressed(KeyCode::Backspace) {
        if !state.is_typing_password { state.username.pop(); } else { state.password.pop(); }
    }

    if keyboard_input.just_pressed(KeyCode::Enter) {
        if state.username.is_empty() || state.password.is_empty() {
            state.error_msg = "Input missing!".to_string();
        } else {
            match state.mode {
                AccountMode::Login => {
                    match database::verify_user(&db_pool.0, &state.username, &state.password) {
                        Ok(true) => {
                            // 【修正】ここでCurrentUserリソースにデータを保存する
                            if let Ok((x, y, words, s, d)) = database::load_user_data(&db_pool.0, &state.username) {
                                // リソースにメモする
                                current_user.username = state.username.clone();
                                current_user.grid_x = x;
                                current_user.grid_y = y;
                                current_user.words = words;

                                // 絵文字はリソースなのでそのまま反映OK
                                emoji_config.s_key = s;
                                emoji_config.d_key = d;

                                notification.message = format!("Welcome back, {}!", state.username);
                                notification.is_visible = true;
                                notification.timer.reset();
                                next_state.set(GameState::Playing);
                            } else {
                                state.error_msg = "Load Error".to_string();
                            }
                        },
                        Ok(false) => state.error_msg = "Invalid User/Pass!".to_string(),
                        Err(e) => state.error_msg = format!("DB Error: {}", e),
                    }
                },
                AccountMode::Create => {
                    match database::user_exists(&db_pool.0, &state.username) {
                        Ok(true) => state.error_msg = "User exists!".to_string(),
                        Ok(false) => {
                            if let Err(e) = database::create_user(&db_pool.0, &state.username, &state.password) {
                                state.error_msg = format!("DB Error: {}", e);
                            } else {
                                if let Ok(mut file) = File::create("credentials.txt") {
                                    let content = format!("ID: {}\nPASS: {}", state.username, state.password);
                                    let _ = file.write_all(content.as_bytes());
                                }
                                
                                // 作成時は初期値
                                current_user.username = state.username.clone();
                                current_user.grid_x = 0;
                                current_user.grid_y = 0;
                                current_user.words = vec!["Hello".to_string(), "Help".to_string(), "Yes".to_string(), "No".to_string()];
                                
                                notification.message = "Account Created!\nSaved to 'credentials.txt'".to_string();
                                notification.is_visible = true;
                                notification.timer.reset();
                                
                                next_state.set(GameState::Playing);
                            }
                        },
                        Err(e) => state.error_msg = format!("DB Err: {}", e),
                    }
                }
            }
        }
    }

    if let Ok(mut text) = text_query.get_single_mut() {
        let (login_tab, create_tab) = match state.mode {
            AccountMode::Login =>  ("[ LOGIN ]", "  SIGN UP  "),
            AccountMode::Create => ("  LOGIN  ", "[ SIGN UP ]"),
        };
        
        let header = format!("{}      {}", login_tab, create_tab);
        let pass_display: String = state.password.chars().map(|_| '*').collect();
        let user_cursor = if !state.is_typing_password { "|" } else { " " };
        let pass_cursor = if state.is_typing_password { "|" } else { " " };

        text.0 = format!(
            "{}\n\nUser: {}{}\nPass: {}{}\n\n{}\n\n[< / >] Switch Mode\n[TAB] Switch Input\n[Enter] Go",
            header,
            state.username, user_cursor,
            pass_display, pass_cursor,
            state.error_msg
        );
    }
}

pub fn cleanup_account_ui(
    mut commands: Commands,
    query: Query<Entity, With<AccountMenuDisplay>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}