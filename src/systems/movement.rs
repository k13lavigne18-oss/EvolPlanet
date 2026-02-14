use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::constants::TILE_SIZE;
use crate::map::is_obstacle;

// ==========================================
// 【CORE LOGIC】 グリッド移動システム
// 役割: 1秒に1回、データ上の位置(GridPosition)だけを更新する。
// 注意: ここで Transform (見た目) は絶対に触らないこと！
// ==========================================
pub fn move_player_tick(
    mut move_timer: ResMut<MoveTimer>,
    time: Res<Time>,
    mut input_buffer: ResMut<InputBuffer>,
    mut query: Query<&mut GridPosition, With<Player>>,
) {
    move_timer.0.tick(time.delta());

    if move_timer.0.finished() {
        if input_buffer.0 != Vec2::ZERO {
            let mut grid_pos = query.single_mut();
            
            let dir = input_buffer.0;
            
            // X軸移動判定
            if dir.x != 0.0 {
                let next_x = grid_pos.x + dir.x as i64;
                if !is_obstacle(next_x, grid_pos.y) {
                    grid_pos.x = next_x;
                }
            }
            
            // Y軸移動判定
            if dir.y != 0.0 {
                let next_y = grid_pos.y + dir.y as i64;
                if !is_obstacle(grid_pos.x, next_y) {
                    grid_pos.y = next_y;
                }
            }

            // 【修正ポイント】
            // ここにあった transform.translation = ... を削除しました。
            // これがあると一瞬でワープしてしまい、補間アニメーションが無効になります。

            // 入力を消費（リセット）して、キーを押し直すまで止まるようにする
            input_buffer.0 = Vec2::ZERO; 
        }
    }
}

// ==========================================
// 【VISUAL LOGIC】 アニメーション同期システム
// 役割: 毎フレーム実行。現在の Transform を GridPosition に向かって滑らかに移動させる。
// ==========================================
pub fn sync_player_pixel_pos(
    time: Res<Time>,
    mut query: Query<(&GridPosition, &mut Transform), With<Player>>,
) {
    let (grid_pos, mut transform) = query.single_mut();

    // 目標とするピクセル座標
    let target_x = grid_pos.x as f32 * TILE_SIZE;
    let target_y = grid_pos.y as f32 * TILE_SIZE;
    // Z座標は現在の値を維持（声のエフェクトなどが隠れないように）
    let target_pos = Vec3::new(target_x, target_y, transform.translation.z);

    // 補間スピード
    // 1.0秒間隔の移動なので、少しゆっくりめ(5.0〜10.0)にすると
    // 「歩いている」感じが出ます。大きくするとキビキビ動きます。
    let smooth_speed = 10.0; 
    let delta = time.delta_secs();

    // Lerp (線形補間) で現在地から目的地へ近づける
    // これにより、GridPositionが切り替わった瞬間に、キャラが「スーッ」と移動します
    transform.translation = transform.translation.lerp(target_pos, delta * smooth_speed);
}