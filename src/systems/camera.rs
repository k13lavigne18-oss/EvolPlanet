use bevy::prelude::*;
use crate::constants::{TILE_SIZE, GRID_COLOR};
use crate::components::Player;

// カメラ追従
pub fn camera_follow(
    player_query: Query<&Transform, (With<Player>, Without<Camera>)>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        if let Ok(mut camera_transform) = camera_query.get_single_mut() {
            // X, Yはプレイヤーに合わせる（Zはそのまま）
            camera_transform.translation.x = player_transform.translation.x;
            camera_transform.translation.y = player_transform.translation.y;
        }
    }
}

// グリッド描画（Gizmosを使用）
pub fn draw_grid_optimized(
    mut gizmos: Gizmos,
    camera_query: Query<(&Camera, &GlobalTransform, &OrthographicProjection)>,
    window_query: Query<&Window>,
) {
    // カメラやウィンドウが取得できない場合は描画しない
    let Ok((_, cam_transform, projection)) = camera_query.get_single() else { return };
    let Ok(window) = window_query.get_single() else { return };

    let view_half_width = window.resolution.width() / 2.0 * projection.scale;
    let view_half_height = window.resolution.height() / 2.0 * projection.scale;
    let cam_pos = cam_transform.translation();

    // 画面内に映る範囲だけ計算
    let start_x = ((cam_pos.x - view_half_width) / TILE_SIZE).floor() as i64;
    let end_x = ((cam_pos.x + view_half_width) / TILE_SIZE).ceil() as i64;
    let start_y = ((cam_pos.y - view_half_height) / TILE_SIZE).floor() as i64;
    let end_y = ((cam_pos.y + view_half_height) / TILE_SIZE).ceil() as i64;

    // グリッドを奥(-10.0)に描画
    let grid_z = -10.0;

    // 縦線
    for x in start_x..=end_x {
        gizmos.line(
            Vec3::new(x as f32 * TILE_SIZE - (TILE_SIZE / 2.0), (start_y as f32 - 1.0) * TILE_SIZE, grid_z),
            Vec3::new(x as f32 * TILE_SIZE - (TILE_SIZE / 2.0), (end_y as f32 + 1.0) * TILE_SIZE, grid_z),
            GRID_COLOR,
        );
    }

    // 横線
    for y in start_y..=end_y {
        gizmos.line(
            Vec3::new((start_x as f32 - 1.0) * TILE_SIZE, y as f32 * TILE_SIZE - (TILE_SIZE / 2.0), grid_z),
            Vec3::new((end_x as f32 + 1.0) * TILE_SIZE, y as f32 * TILE_SIZE - (TILE_SIZE / 2.0), grid_z),
            GRID_COLOR,
        );
    }
}