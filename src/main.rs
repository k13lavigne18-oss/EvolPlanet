mod constants;
mod components;
mod resources;
mod map;
mod systems;
mod database;

use bevy::prelude::*;
use std::time::Duration;

use resources::*;
use systems::startup::*;
use systems::input::*;
use systems::movement::*;
use systems::camera::*;
use systems::ui::*;
use systems::map_render::*;
use systems::account::*;
use systems::bot::*; 
use constants::PLAYER_MOVE_INTERVAL;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::WHITE))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Evolution: Origin".into(),
                resolution: (800., 600.).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        
        .insert_resource(MoveTimer::new(Timer::new(Duration::from_secs_f32(PLAYER_MOVE_INTERVAL), TimerMode::Repeating)))
        
        .insert_resource(InputBuffer(Vec2::ZERO))
        .insert_resource(ChatLog { messages: Vec::new() })
        
        .insert_resource(EmojiConfig {
            s_key: "😁".to_string(),
            d_key: "😭".to_string(),
        })
        .insert_resource(EmojiSelectState {
            is_open: false,
            target_key: None,
            selected_index: 0,
        })
        .insert_resource(ChatMenuState {
            is_open: false,
            selected_index: 0,
        })
        
        .insert_resource(AccountState {
            mode: AccountMode::Login,
            username: "".to_string(),
            password: "".to_string(),
            is_typing_password: false,
            error_msg: "".to_string(),
        })
        
        .insert_resource(NotificationState {
            message: "".to_string(),
            timer: Timer::from_seconds(10.0, TimerMode::Once),
            is_visible: false,
        })
        
        .insert_resource(CurrentUser::default())
        
        // 【新規】BotDialoguesを初期化
        .insert_resource(BotDialogues::default())
        
        .add_systems(Startup, setup)
        
        .add_systems(OnEnter(GameState::Login), setup_account_ui)
        .add_systems(Update, handle_account_input.run_if(in_state(GameState::Login)))
        .add_systems(OnExit(GameState::Login), cleanup_account_ui)

        .add_systems(OnEnter(GameState::Playing), setup_game)

        .add_systems(Update, (
            handle_movement_input,
            handle_chat_input,
            
            move_player_tick,
            sync_player_pixel_pos,
            
            camera_follow,
            draw_grid_optimized,
            spawn_visible_obstacles,
            
            spawn_visible_bots,
            despawn_far_bots,
            bot_wander_system,
            update_bot_chat,
            
            update_ui,
            update_chat_ui, 
            update_chat_menu_ui,
            update_emoji_select_menu,
            update_notification_ui,
            handle_save_button_interaction,
        ).run_if(in_state(GameState::Playing)))
        
        .run();
}