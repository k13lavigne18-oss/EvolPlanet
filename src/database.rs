use r2d2::Pool;
use r2d2_postgres::PostgresConnectionManager;
use postgres::NoTls;

// DB初期化（テーブル作成）
pub fn init_db(pool: &Pool<PostgresConnectionManager<NoTls>>) -> Result<(), String> {
    let mut client = pool.get().map_err(|e| e.to_string())?;
    
    // 【変更】座標、言葉、絵文字設定を保存できるカラムを追加
    // words は TEXT[] (文字列の配列) として保存します
    client.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            username VARCHAR(50) NOT NULL UNIQUE,
            password VARCHAR(100) NOT NULL,
            grid_x BIGINT DEFAULT 0,
            grid_y BIGINT DEFAULT 0,
            words TEXT[],
            s_key VARCHAR(10) DEFAULT '😁',
            d_key VARCHAR(10) DEFAULT '😭'
        )",
        &[],
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}

// ユーザー作成 (初期データも登録)
pub fn create_user(pool: &Pool<PostgresConnectionManager<NoTls>>, username: &str, password: &str) -> Result<(), String> {
    let mut client = pool.get().map_err(|e| e.to_string())?;
    
    // 初期状態の言葉リスト
    let initial_words: Vec<String> = vec![
        "Hello".to_string(), "Help".to_string(), "Yes".to_string(), "No".to_string()
    ];

    client.execute(
        "INSERT INTO users (username, password, grid_x, grid_y, words, s_key, d_key) 
         VALUES ($1, $2, 0, 0, $3, '😁', '😭')",
        &[&username, &password, &initial_words],
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}

// ユーザー存在チェック
pub fn user_exists(pool: &Pool<PostgresConnectionManager<NoTls>>, username: &str) -> Result<bool, String> {
    let mut client = pool.get().map_err(|e| e.to_string())?;
    let row = client.query_one("SELECT count(*) FROM users WHERE username = $1", &[&username]).map_err(|e| e.to_string())?;
    let count: i64 = row.get(0);
    Ok(count > 0)
}

// ログインチェック (成功したら true)
pub fn verify_user(pool: &Pool<PostgresConnectionManager<NoTls>>, username: &str, password: &str) -> Result<bool, String> {
    let mut client = pool.get().map_err(|e| e.to_string())?;
    let row = client.query_one(
        "SELECT count(*) FROM users WHERE username = $1 AND password = $2",
        &[&username, &password],
    ).map_err(|e| e.to_string())?;
    let count: i64 = row.get(0);
    Ok(count > 0)
}

// 【新規】データのロード
// 戻り値: (x, y, words, s_key, d_key)
pub fn load_user_data(pool: &Pool<PostgresConnectionManager<NoTls>>, username: &str) -> Result<(i64, i64, Vec<String>, String, String), String> {
    let mut client = pool.get().map_err(|e| e.to_string())?;
    
    let row = client.query_one(
        "SELECT grid_x, grid_y, words, s_key, d_key FROM users WHERE username = $1",
        &[&username],
    ).map_err(|e| e.to_string())?;

    let x: i64 = row.get(0);
    let y: i64 = row.get(1);
    let words: Vec<String> = row.get(2);
    let s_key: String = row.get(3);
    let d_key: String = row.get(4);

    Ok((x, y, words, s_key, d_key))
}

// 【新規】データのセーブ
pub fn save_user_data(
    pool: &Pool<PostgresConnectionManager<NoTls>>, 
    username: &str, 
    x: i64, 
    y: i64, 
    words: Vec<String>, 
    s_key: &str, 
    d_key: &str
) -> Result<(), String> {
    let mut client = pool.get().map_err(|e| e.to_string())?;
    
    client.execute(
        "UPDATE users SET grid_x = $1, grid_y = $2, words = $3, s_key = $4, d_key = $5 WHERE username = $6",
        &[&x, &y, &words, &s_key, &d_key, &username],
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}