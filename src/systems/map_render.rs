use bevy::prelude::*;
use crate::constants::TILE_SIZE;
use crate::map::is_obstacle;
// 未使用のインポートを削除しました

#[derive(Component)]
pub struct Obstacle;

pub fn spawn_visible_obstacles(
    mut commands: Commands,
    camera_query: Query<(&Camera, &GlobalTransform, &OrthographicProjection)>,
    window_query: Query<&Window>,
    obstacle_query: Query<Entity, With<Obstacle>>,
) {
    // camera 変数は使わないので _ に変更
    let (_, cam_transform, projection) = camera_query.single();
    let window = window_query.single();

    for entity in &obstacle_query {
        commands.entity(entity).despawn();
    }

    let view_half_width = window.resolution.width() / 2.0 * projection.scale;
    let view_half_height = window.resolution.height() / 2.0 * projection.scale;
    let cam_pos = cam_transform.translation();

    let buffer = 1.0; 
    let start_x = ((cam_pos.x - view_half_width) / TILE_SIZE - buffer).floor() as i64;
    let end_x = ((cam_pos.x + view_half_width) / TILE_SIZE + buffer).ceil() as i64;
    let start_y = ((cam_pos.y - view_half_height) / TILE_SIZE - buffer).floor() as i64;
    let end_y = ((cam_pos.y + view_half_height) / TILE_SIZE + buffer).ceil() as i64;

    for x in start_x..=end_x {
        for y in start_y..=end_y {
            if is_obstacle(x, y) {
                commands.spawn((
                    Obstacle,
                    Sprite {
                        color: Color::srgb(0.5, 0.5, 0.5),
                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                        ..default()
                    },
                    Transform::from_xyz(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE, 0.0),
                ));
            }
        }
    }
}