use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::constants::TILE_SIZE;
use rand::seq::SliceRandom; // ランダムピック用

// 移動入力システム
pub fn handle_movement_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut input_buffer: ResMut<InputBuffer>,
    emoji_state: Res<EmojiSelectState>,
    chat_menu_state: Res<ChatMenuState>,
) {
    if emoji_state.is_open || chat_menu_state.is_open {
        input_buffer.0 = Vec2::ZERO;
        return;
    }

    let mut direction = Vec2::ZERO;
    if keyboard_input.pressed(KeyCode::ArrowUp) { direction.y += 1.0; }
    if keyboard_input.pressed(KeyCode::ArrowDown) { direction.y -= 1.0; }
    if keyboard_input.pressed(KeyCode::ArrowLeft) { direction.x -= 1.0; }
    if keyboard_input.pressed(KeyCode::ArrowRight) { direction.x += 1.0; }

    if direction != Vec2::ZERO {
        input_buffer.0 = direction;
    }
}

// チャット・絵文字入力システム
pub fn handle_chat_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut chat_log: ResMut<ChatLog>,
    player_query: Query<&Vocabulary, With<Player>>,
    mut emoji_query: Query<(&mut Text, &mut EmojiTimer), With<PlayerEmoji>>,
    
    mut emoji_config: ResMut<EmojiConfig>,
    mut emoji_state: ResMut<EmojiSelectState>,

    mut voice_query: Query<&mut Visibility, With<VoiceEffect>>,
    
    // 【修正】BotDialogues リソースを追加
    bot_dialogues: Res<BotDialogues>,
    
    mut bot_query: Query<(&Transform, &Children, &mut BotTalking), With<Bot>>,
    mut bot_text_query: Query<(&mut Text2d, &mut BotChatTimer), With<BotChatText>>,
    
    player_transform_query: Query<&Transform, With<Player>>,
) {
    if emoji_state.is_open {
        let total_count = crate::resources::EMOJI_LIST.len();
        if keyboard_input.just_pressed(KeyCode::ArrowUp) {
            if emoji_state.selected_index > 0 { emoji_state.selected_index -= 1; }
        }
        if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            if emoji_state.selected_index < total_count.saturating_sub(1) { emoji_state.selected_index += 1; }
        }
        if keyboard_input.just_pressed(KeyCode::Enter) {
            let selected_emoji = crate::resources::EMOJI_LIST[emoji_state.selected_index];
            match emoji_state.target_key {
                Some(KeyCode::KeyS) => emoji_config.s_key = selected_emoji.to_string(),
                Some(KeyCode::KeyD) => emoji_config.d_key = selected_emoji.to_string(),
                _ => {}
            }
            emoji_state.is_open = false;
            emoji_state.target_key = None;
        }
        if keyboard_input.just_pressed(KeyCode::Escape) {
            emoji_state.is_open = false;
            emoji_state.target_key = None;
        }
        return; 
    }

    if keyboard_input.just_pressed(KeyCode::KeyA) {
        if let Ok((mut text, mut timer)) = emoji_query.get_single_mut() {
            text.0 = "👍".to_string();
            timer.0.reset();
        }
    }
    if keyboard_input.just_pressed(KeyCode::KeyS) {
        if keyboard_input.pressed(KeyCode::Space) {
            emoji_state.is_open = true;
            emoji_state.target_key = Some(KeyCode::KeyS);
            emoji_state.selected_index = 0;
        } else {
             if let Ok((mut text, mut timer)) = emoji_query.get_single_mut() {
                text.0 = emoji_config.s_key.clone();
                timer.0.reset();
            }
        }
    }
    if keyboard_input.just_pressed(KeyCode::KeyD) {
        if keyboard_input.pressed(KeyCode::Space) {
            emoji_state.is_open = true;
            emoji_state.target_key = Some(KeyCode::KeyD);
            emoji_state.selected_index = 0;
        } else {
             if let Ok((mut text, mut timer)) = emoji_query.get_single_mut() {
                text.0 = emoji_config.d_key.clone();
                timer.0.reset();
            }
        }
    }

    if let Ok(vocab) = player_query.get_single() {
        let mut selected_index = None;
        if keyboard_input.just_pressed(KeyCode::Digit1) { selected_index = Some(0); }
        if keyboard_input.just_pressed(KeyCode::Digit2) { selected_index = Some(1); }
        if keyboard_input.just_pressed(KeyCode::Digit3) { selected_index = Some(2); }
        if keyboard_input.just_pressed(KeyCode::Digit4) { selected_index = Some(3); }

        if let Some(index) = selected_index {
            if index < vocab.words.len() {
                let word = &vocab.words[index];
                
                chat_log.messages.push((
                    format!("> {}", word),
                    Timer::new(std::time::Duration::from_secs(5), TimerMode::Once)
                ));

                for mut visibility in &mut voice_query {
                    *visibility = Visibility::Visible;
                }

                if let Ok(p_transform) = player_transform_query.get_single() {
                    let range = TILE_SIZE * 4.5; 

                    for (b_transform, children, mut talking) in &mut bot_query {
                        let dist = p_transform.translation.distance(b_transform.translation);
                        
                        if dist <= range {
                            // 【修正】ランダムピック処理
                            let response = if word == "Hello" {
                                "Hello".to_string()
                            } else {
                                // リソースからランダムに1行選ぶ
                                if let Some(line) = bot_dialogues.lines.choose(&mut rand::thread_rng()) {
                                    line.clone()
                                } else {
                                    "...".to_string() // ファイルが空の場合
                                }
                            };

                            talking.0.set_duration(std::time::Duration::from_secs(3));
                            talking.0.reset();

                            for &child in children.iter() {
                                if let Ok((mut text, mut timer)) = bot_text_query.get_mut(child) {
                                    text.0 = response.clone(); 
                                    timer.0.reset();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}