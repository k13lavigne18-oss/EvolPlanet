use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::database;
use crate::components::SaveButton;

pub fn update_ui(
    player_query: Query<&GridPosition, With<Player>>,
    input_buffer: Res<InputBuffer>,
    mut pos_text_query: Query<&mut Text, (With<PositionText>, Without<DirectionText>)>,
    mut dir_text_query: Query<&mut Text, (With<DirectionText>, Without<PositionText>)>,
) {
    if let Ok(pos) = player_query.get_single() {
        if let Ok(mut text) = pos_text_query.get_single_mut() {
            text.0 = format!("({}, {})", pos.x, pos.y);
        }
    }

    if let Ok(mut text) = dir_text_query.get_single_mut() {
        let dir = input_buffer.0;
        let symbol = if dir.y > 0.5 { "^" }
        else if dir.y < -0.5 { "v" }
        else if dir.x > 0.5 { ">" }
        else if dir.x < -0.5 { "<" }
        else { "." };
        text.0 = symbol.to_string();
    }
}

pub fn update_chat_ui(
    time: Res<Time>,
    mut chat_log: ResMut<ChatLog>,
    mut chat_text_query: Query<&mut Text, With<ChatDisplay>>,
    mut emoji_query: Query<(&mut Text, &mut EmojiTimer), (With<PlayerEmoji>, Without<ChatDisplay>)>,
    // 【新規】声のエフェクト用
    mut voice_query: Query<&mut Visibility, With<VoiceEffect>>,
) {
    // ログの寿命管理
    chat_log.messages.retain_mut(|(_, timer)| {
        timer.tick(time.delta());
        !timer.finished()
    });

    // 【新規】ログが空なら声のエフェクトを消す
    if chat_log.messages.is_empty() {
        for mut visibility in &mut voice_query {
            *visibility = Visibility::Hidden;
        }
    } else {
        // ログがある間は表示
        for mut visibility in &mut voice_query {
            *visibility = Visibility::Visible;
        }
    }

    // チャットログ表示更新
    let start_index = if chat_log.messages.len() > 5 {
        chat_log.messages.len() - 5
    } else {
        0
    };

    if let Ok(mut text) = chat_text_query.get_single_mut() {
        let display_string = chat_log.messages[start_index..]
            .iter()
            .map(|(msg, _)| msg.as_str())
            .collect::<Vec<&str>>()
            .join("\n");
        text.0 = display_string;
    }

    // 絵文字の消去タイマー
    if let Ok((mut text, mut timer)) = emoji_query.get_single_mut() {
        if !text.0.is_empty() {
            timer.0.tick(time.delta());
            if timer.0.finished() {
                text.0 = "".to_string();
            }
        }
    }
}

pub fn update_chat_menu_ui(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Vocabulary, With<Player>>,
    mut menu_query: Query<(&mut Text, &mut Node), With<ChatMenuDisplay>>,
    emoji_config: Res<EmojiConfig>,
    mut menu_state: ResMut<ChatMenuState>,
) {
    let (mut text, mut node) = menu_query.single_mut();
    
    if keyboard_input.pressed(KeyCode::Space) {
        node.display = Display::Flex;
        menu_state.is_open = true;
    } else {
        node.display = Display::None;
        menu_state.is_open = false;
        return;
    }

    let Ok(vocab) = player_query.get_single() else { return };
    let total_count = vocab.words.len();
    
    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        if menu_state.selected_index > 0 {
            menu_state.selected_index -= 1;
        }
    }
    if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        if menu_state.selected_index < total_count.saturating_sub(1) {
            menu_state.selected_index += 1;
        }
    }

    let mut menu_str = String::new();
    menu_str.push_str("Available Words:\n");

    let visible_count = 5;
    let start = menu_state.selected_index;
    let end = (start + visible_count).min(total_count);

    if start > 0 {
        menu_str.push_str("  (▲ up)\n");
    }

    for i in start..end {
        let word = &vocab.words[i];
        let prefix = if i < 4 {
            format!("[{}]", i + 1)
        } else {
            format!(" - ")
        };
        menu_str.push_str(&format!("{} {}\n", prefix, word));
    }

    if end < total_count {
        menu_str.push_str("  (▼ down)\n");
    }
    
    menu_str.push_str("\nEmotes:\n");
    menu_str.push_str("[A] 👍\n");
    menu_str.push_str(&format!("[S] {}\n", emoji_config.s_key));
    menu_str.push_str(&format!("[D] {}\n", emoji_config.d_key));

    text.0 = menu_str;
}

pub fn update_emoji_select_menu(
    emoji_state: Res<EmojiSelectState>,
    mut query: Query<(&mut Text, &mut Node), With<EmojiSelectMenuDisplay>>,
) {
    let (mut text, mut node) = query.single_mut();

    if !emoji_state.is_open {
        node.display = Display::None;
        return;
    }
    node.display = Display::Flex;

    let target_str = match emoji_state.target_key {
        Some(KeyCode::KeyS) => "S Key",
        Some(KeyCode::KeyD) => "D Key",
        _ => "Unknown",
    };

    let mut content = format!("Select Emoji for [{}]:\n\n", target_str);
    
    let visible_count = 7;
    let total_count = EMOJI_LIST.len();
    
    let start_index = if emoji_state.selected_index < visible_count / 2 {
        0
    } else if emoji_state.selected_index > total_count - visible_count / 2 {
        if total_count > visible_count { total_count - visible_count } else { 0 }
    } else {
        emoji_state.selected_index - visible_count / 2
    };

    let end_index = (start_index + visible_count).min(total_count);

    if start_index > 0 { content.push_str("  ... (more) ...\n"); }

    for i in start_index..end_index {
        let emoji = EMOJI_LIST[i];
        let cursor = if i == emoji_state.selected_index { ">" } else { " " };
        content.push_str(&format!("{} {}\n", cursor, emoji));
    }

    if end_index < total_count { content.push_str("  ... (more) ...\n"); }
    
    content.push_str("\n[Enter] Select");
    text.0 = content;
}

pub fn update_notification_ui(
    time: Res<Time>,
    mut state: ResMut<NotificationState>,
    mut query: Query<(&mut Node, &Children), With<NotificationDisplay>>,
    mut text_query: Query<&mut Text, With<NotificationText>>,
) {
    let (mut node, _children) = query.single_mut();

    if state.is_visible {
        node.display = Display::Flex;
        state.timer.tick(time.delta());
        if state.timer.finished() {
            state.is_visible = false;
        }
        for mut text in text_query.iter_mut() {
            text.0 = state.message.clone();
        }
    } else {
        node.display = Display::None;
    }
}

pub fn handle_save_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<SaveButton>)>,
    current_user: Res<CurrentUser>,
    player_query: Query<(&GridPosition, &Vocabulary), With<Player>>,
    emoji_config: Res<EmojiConfig>,
    db_pool: Res<DbPool>,
    mut notification: ResMut<NotificationState>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            if current_user.username.is_empty() { return; }

            if let Ok((pos, vocab)) = player_query.get_single() {
                match database::save_user_data(
                    &db_pool.0,
                    &current_user.username,
                    pos.x,
                    pos.y,
                    vocab.words.clone(),
                    &emoji_config.s_key,
                    &emoji_config.d_key,
                ) {
                    Ok(_) => {
                        notification.message = "Game Saved!".to_string();
                        notification.is_visible = true;
                        notification.timer.reset();
                    },
                    Err(e) => {
                        notification.message = format!("Save Error: {}", e);
                        notification.is_visible = true;
                        notification.timer.reset();
                    }
                }
            }
        }
    }
}