// 障害物の出現ロジック
const OBSTACLE_DENSITY: u64 = 3; 

pub fn is_obstacle(x: i64, y: i64) -> bool {
    if x == 0 && y == 0 { return false; }

    let mut h = (x as u128).wrapping_mul(0x9E3779B97F4A7C15);
    h = h.wrapping_add((y as u128).wrapping_mul(0xBF58476D1CE4E5B9));
    h = (h ^ (h >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    h = (h ^ (h >> 27)).wrapping_mul(0x94D049BB133111EB);
    h = h ^ (h >> 31);
    
    (h % 100) < OBSTACLE_DENSITY as u128
}

// 【修正】ボットの出現ロジック
// 2億体目標とのことですが、まずは「見つかること」を優先し
// 200セルに1体 (0.5%) 程度の密度にします。
const BOT_DENSITY: u64 = 1; 
const BOT_MODULO: u64 = 200; // ここを小さくすると密度が上がる

pub fn is_bot_spawn(x: i64, y: i64) -> bool {
    // スタート地点付近はボットなし
    if x.abs() < 10 && y.abs() < 10 { return false; }

    if is_obstacle(x, y) { return false; }

    let mut h = (x as u64).wrapping_mul(0x9E3779B97F4A7C15);
    h = (h ^ (y as u64)).wrapping_mul(0xBF58476D1CE4E5B9);
    h = (h ^ (h >> 27)).wrapping_mul(0x94D049BB133111EB);
    h = h ^ (h >> 31);
    
    (h % BOT_MODULO) < BOT_DENSITY
}