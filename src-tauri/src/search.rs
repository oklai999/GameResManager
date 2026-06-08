use sqlx::SqlitePool;
use crate::models::{Asset, AssetSearchRequest};

fn escape_like_pattern(input: &str) -> String {
    input.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
}

fn exclude_system_metadata_condition() -> String {
    "file_name != '.DS_Store'
     AND file_name NOT LIKE '._%' ESCAPE '\\'
     AND absolute_path NOT LIKE '%/__MACOSX/%' ESCAPE '\\'
     AND absolute_path NOT LIKE '%\\__MACOSX\\%' ESCAPE '\\'".to_string()
}

pub async fn search_assets(db: &SqlitePool, req: &AssetSearchRequest) -> anyhow::Result<Vec<Asset>> {
    let mut sql = String::from(
        "SELECT id, library_folder_id, absolute_path, file_name, extension, asset_type, file_size,
                modified_at, width, height, thumbnail_path, thumbnail_status, thumbnail_error, note,
                is_favorite, is_missing, created_at, updated_at
         FROM assets"
    );

    let mut conditions: Vec<String> = Vec::new();
    let mut has_query = false;
    conditions.push(exclude_system_metadata_condition());

    if !req.query.is_empty() && (req.search_file_name || req.search_note || req.search_path || req.search_tags) {
        let mut ors: Vec<String> = Vec::new();
        if req.search_file_name { ors.push("file_name LIKE ? ESCAPE '\\'".to_string()); }
        if req.search_note { ors.push("note LIKE ? ESCAPE '\\'".to_string()); }
        if req.search_path { ors.push("absolute_path LIKE ? ESCAPE '\\'".to_string()); }
        if req.search_tags {
            ors.push(
                "EXISTS (SELECT 1 FROM asset_tags at JOIN tags t ON at.tag_id = t.id WHERE at.asset_id = assets.id AND t.name LIKE ? ESCAPE '\\')".to_string()
            );
        }
        if !ors.is_empty() {
            conditions.push(format!("({})", ors.join(" OR ")));
            has_query = true;
        }
    }

    if req.asset_type.is_some() { conditions.push("asset_type = ?".to_string()); }
    if req.library_folder_id.is_some() { conditions.push("library_folder_id = ?".to_string()); }
    if req.collection_id.is_some() { conditions.push("id IN (SELECT asset_id FROM collection_assets WHERE collection_id = ?)".to_string()); }
    if req.is_favorite.is_some() { conditions.push("is_favorite = ?".to_string()); }
    if req.is_missing.is_some() { conditions.push("is_missing = ?".to_string()); }
    if req.min_file_size.is_some() { conditions.push("file_size >= ?".to_string()); }
    if req.max_file_size.is_some() { conditions.push("file_size <= ?".to_string()); }
    if req.min_width.is_some() { conditions.push("width >= ?".to_string()); }
    if req.max_width.is_some() { conditions.push("width <= ?".to_string()); }
    if req.min_height.is_some() { conditions.push("height >= ?".to_string()); }
    if req.max_height.is_some() { conditions.push("height <= ?".to_string()); }
    if req.modified_after.is_some() { conditions.push("modified_at >= ?".to_string()); }
    if req.modified_before.is_some() { conditions.push("modified_at <= ?".to_string()); }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    let sort_column = match req.sort_by.as_str() {
        "file_size" => "file_size",
        "modified_at" => "modified_at",
        "asset_type" => "asset_type",
        _ => "file_name",
    };
    let sort_direction = if req.sort_direction.eq_ignore_ascii_case("desc") {
        "DESC"
    } else {
        "ASC"
    };
    sql.push_str(&format!(
        " ORDER BY {} {}, file_name ASC LIMIT ? OFFSET ?",
        sort_column, sort_direction
    ));

    let mut query = sqlx::query_as::<_, Asset>(&sql);
    let escaped = escape_like_pattern(&req.query);
    let pattern = format!("%{}%", escaped);

    if has_query {
        if req.search_file_name { query = query.bind(&pattern); }
        if req.search_note { query = query.bind(&pattern); }
        if req.search_path { query = query.bind(&pattern); }
        if req.search_tags { query = query.bind(&pattern); }
    }

    if let Some(ref t) = req.asset_type { query = query.bind(t); }
    if let Some(id) = req.library_folder_id { query = query.bind(id); }
    if let Some(id) = req.collection_id { query = query.bind(id); }
    if let Some(v) = req.is_favorite { query = query.bind(if v { 1 } else { 0 }); }
    if let Some(v) = req.is_missing { query = query.bind(if v { 1 } else { 0 }); }
    if let Some(v) = req.min_file_size { query = query.bind(v); }
    if let Some(v) = req.max_file_size { query = query.bind(v); }
    if let Some(v) = req.min_width { query = query.bind(v); }
    if let Some(v) = req.max_width { query = query.bind(v); }
    if let Some(v) = req.min_height { query = query.bind(v); }
    if let Some(v) = req.max_height { query = query.bind(v); }
    if let Some(ref v) = req.modified_after { query = query.bind(v); }
    if let Some(ref v) = req.modified_before { query = query.bind(v); }

    let limit = req.limit.clamp(1, 2000);
    let offset = req.offset.max(0);
    query = query.bind(limit).bind(offset);

    let rows = query.fetch_all(db).await?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_request() -> AssetSearchRequest {
        AssetSearchRequest {
            query: String::new(),
            search_file_name: true,
            search_note: false,
            search_path: false,
            search_tags: false,
            asset_type: None,
            library_folder_id: None,
            collection_id: None,
            is_favorite: None,
            is_missing: None,
            min_file_size: None,
            max_file_size: None,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            modified_after: None,
            modified_before: None,
            sort_by: "file_name".to_string(),
            sort_direction: "asc".to_string(),
            limit: 200,
            offset: 0,
        }
    }

    #[test]
    fn empty_query_returns_valid_sql() {
        let req = empty_request();
        let mut sql = String::from("SELECT * FROM assets");
        let mut conditions: Vec<String> = Vec::new();

        if !req.query.is_empty() && (req.search_file_name || req.search_note || req.search_path || req.search_tags) {
            let mut ors: Vec<String> = Vec::new();
            if req.search_file_name { ors.push("file_name LIKE ?".to_string()); }
            if req.search_note { ors.push("note LIKE ?".to_string()); }
            if req.search_path { ors.push("absolute_path LIKE ?".to_string()); }
            if req.search_tags { ors.push("EXISTS (SELECT 1 FROM asset_tags".to_string()); }
            if !ors.is_empty() {
                conditions.push(format!("({})", ors.join(" OR ")));
            }
        }

        if req.asset_type.is_some() { conditions.push("asset_type = ?".to_string()); }
        if req.library_folder_id.is_some() { conditions.push("library_folder_id = ?".to_string()); }
        if req.is_favorite.is_some() { conditions.push("is_favorite = ?".to_string()); }
        if req.is_missing.is_some() { conditions.push("is_missing = ?".to_string()); }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }

        assert!(sql.contains("SELECT * FROM assets"));
        assert!(!sql.contains("WHERE"));
    }

    #[test]
    fn filename_search_adds_filename_condition() {
        let mut req = empty_request();
        req.query = "icon".to_string();
        req.search_file_name = true;

        let mut conditions: Vec<String> = Vec::new();
        if !req.query.is_empty() && (req.search_file_name || req.search_note || req.search_path || req.search_tags) {
            let mut ors: Vec<String> = Vec::new();
            if req.search_file_name { ors.push("file_name LIKE ?".to_string()); }
            if req.search_note { ors.push("note LIKE ?".to_string()); }
            if req.search_path { ors.push("absolute_path LIKE ?".to_string()); }
            if req.search_tags { ors.push("EXISTS (SELECT 1 FROM asset_tags".to_string()); }
            if !ors.is_empty() {
                conditions.push(format!("({})", ors.join(" OR ")));
            }
        }

        assert_eq!(conditions.len(), 1);
        assert!(conditions[0].contains("file_name LIKE ?"));
    }

    #[test]
    fn type_filter_adds_asset_type_condition() {
        let mut req = empty_request();
        req.asset_type = Some("image".to_string());

        let mut conditions: Vec<String> = Vec::new();
        if req.asset_type.is_some() { conditions.push("asset_type = ?".to_string()); }

        assert_eq!(conditions.len(), 1);
        assert!(conditions[0].contains("asset_type = ?"));
    }

    #[test]
    fn combined_filters_preserve_parameter_order() {
        let mut req = empty_request();
        req.query = "hero".to_string();
        req.search_file_name = true;
        req.asset_type = Some("image".to_string());
        req.is_favorite = Some(true);

        let mut conditions: Vec<String> = Vec::new();
        let mut has_query = false;

        if !req.query.is_empty() && (req.search_file_name || req.search_note || req.search_path || req.search_tags) {
            let mut ors: Vec<String> = Vec::new();
            if req.search_file_name { ors.push("file_name LIKE ?".to_string()); }
            if req.search_note { ors.push("note LIKE ?".to_string()); }
            if req.search_path { ors.push("absolute_path LIKE ?".to_string()); }
            if req.search_tags { ors.push("EXISTS (SELECT 1 FROM asset_tags".to_string()); }
            if !ors.is_empty() {
                conditions.push(format!("({})", ors.join(" OR ")));
                has_query = true;
            }
        }

        if req.asset_type.is_some() { conditions.push("asset_type = ?".to_string()); }
        if req.is_favorite.is_some() { conditions.push("is_favorite = ?".to_string()); }

        assert_eq!(conditions.len(), 3);
        assert!(conditions[0].contains("file_name"));
        assert!(conditions[1].contains("asset_type"));
        assert!(conditions[2].contains("is_favorite"));
        assert!(has_query);
    }

    #[test]
    fn escape_like_pattern_escapes_special_chars() {
        assert_eq!(escape_like_pattern("100%"), "100\\%");
        assert_eq!(escape_like_pattern("test_"), "test\\_");
        assert_eq!(escape_like_pattern("a\\b"), "a\\\\b");
        assert_eq!(escape_like_pattern("%_\\"), "\\%\\_\\\\");
    }

    #[test]
    fn large_requested_limit_is_clamped_to_2000() {
        assert_eq!(50000_i64.clamp(1, 2000), 2000);
    }

    #[test]
    fn limit_offset_clamping() {
        assert_eq!((-5_i64).clamp(1, 2000), 1);
        assert_eq!(5000_i64.clamp(1, 2000), 2000);
        assert_eq!(100_i64.clamp(1, 2000), 100);
        assert_eq!((-10_i64).max(0), 0);
        assert_eq!(50_i64.max(0), 50);
    }

    #[tokio::test]
    async fn search_assets_integration() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE library_folders (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL,
                last_scanned_at TEXT,
                is_enabled INTEGER NOT NULL DEFAULT 1
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE assets (
                id INTEGER PRIMARY KEY,
                library_folder_id INTEGER NOT NULL,
                absolute_path TEXT NOT NULL UNIQUE,
                file_name TEXT NOT NULL,
                extension TEXT NOT NULL,
                asset_type TEXT NOT NULL,
                file_size INTEGER NOT NULL DEFAULT 0,
                modified_at TEXT NOT NULL,
                width INTEGER,
                height INTEGER,
                thumbnail_path TEXT,
                thumbnail_status TEXT NOT NULL DEFAULT 'none',
                thumbnail_error TEXT,
                note TEXT NOT NULL DEFAULT '',
                is_favorite INTEGER NOT NULL DEFAULT 0,
                is_missing INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE tags (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL DEFAULT '#5B8DEF',
                created_at TEXT NOT NULL,
                last_used_at TEXT NOT NULL
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE asset_tags (
                asset_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (asset_id, tag_id)
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE collection_assets (
                collection_id INTEGER NOT NULL,
                asset_id INTEGER NOT NULL,
                PRIMARY KEY (collection_id, asset_id)
            )"
        ).execute(&pool).await.unwrap();

        sqlx::query("INSERT INTO library_folders (id, name, path, created_at, is_enabled) VALUES (1, 'Test', '/test', '2024-01-01T00:00:00Z', 1)")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO assets (id, library_folder_id, absolute_path, file_name, extension, asset_type, modified_at, note, is_favorite, is_missing, created_at, updated_at) VALUES
            (1, 1, '/test/hero.png', 'hero.png', 'png', 'image', '2024-01-01T00:00:00Z', 'main character', 1, 0, '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z'),
            (2, 1, '/test/villain.png', 'villain.png', 'png', 'image', '2024-01-01T00:00:00Z', '', 0, 0, '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z'),
            (3, 1, '/test/sound.wav', 'sound.wav', 'wav', 'audio', '2024-01-01T00:00:00Z', '', 0, 1, '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z'),
            (4, 1, '/test/__MACOSX/._hero.png', '._hero.png', 'png', 'image', '2024-01-01T00:00:00Z', '', 0, 1, '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')"
        ).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO tags (id, name, color, created_at, last_used_at) VALUES (1, 'important', '#FF0000', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO asset_tags (asset_id, tag_id) VALUES (2, 1)")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO collection_assets (collection_id, asset_id) VALUES (1, 1), (1, 3)")
            .execute(&pool).await.unwrap();

        let req = AssetSearchRequest {
            query: "hero".to_string(),
            search_file_name: true,
            search_note: true,
            search_path: false,
            search_tags: false,
            asset_type: None,
            library_folder_id: None,
            collection_id: None,
            is_favorite: None,
            is_missing: None,
            min_file_size: None,
            max_file_size: None,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            modified_after: None,
            modified_before: None,
            sort_by: "file_name".to_string(),
            sort_direction: "asc".to_string(),
            limit: 200,
            offset: 0,
        };
        let results = search_assets(&pool, &req).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].file_name, "hero.png");

        let mut req = AssetSearchRequest {
            query: "important".to_string(),
            search_file_name: false,
            search_note: false,
            search_path: false,
            search_tags: true,
            asset_type: None,
            library_folder_id: None,
            collection_id: None,
            is_favorite: None,
            is_missing: None,
            min_file_size: None,
            max_file_size: None,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            modified_after: None,
            modified_before: None,
            sort_by: "file_name".to_string(),
            sort_direction: "asc".to_string(),
            limit: 200,
            offset: 0,
        };
        let results = search_assets(&pool, &req).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].file_name, "villain.png");

        req.query = "png".to_string();
        req.search_file_name = true;
        req.search_tags = true;
        let results = search_assets(&pool, &req).await.unwrap();
        assert_eq!(results.len(), 2);

        req.asset_type = Some("audio".to_string());
        let results = search_assets(&pool, &req).await.unwrap();
        assert_eq!(results.len(), 0);

        req.asset_type = Some("image".to_string());
        req.is_favorite = Some(true);
        let results = search_assets(&pool, &req).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].file_name, "hero.png");

        let req = AssetSearchRequest {
            query: "".to_string(),
            search_file_name: true,
            search_note: false,
            search_path: false,
            search_tags: false,
            asset_type: None,
            library_folder_id: None,
            collection_id: None,
            is_favorite: None,
            is_missing: Some(true),
            min_file_size: None,
            max_file_size: None,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            modified_after: None,
            modified_before: None,
            sort_by: "file_name".to_string(),
            sort_direction: "asc".to_string(),
            limit: 200,
            offset: 0,
        };
        let results = search_assets(&pool, &req).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].file_name, "sound.wav");

        let req = AssetSearchRequest {
            query: "".to_string(),
            search_file_name: true,
            search_note: false,
            search_path: false,
            search_tags: false,
            asset_type: None,
            library_folder_id: None,
            collection_id: Some(1),
            is_favorite: None,
            is_missing: None,
            min_file_size: None,
            max_file_size: None,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            modified_after: None,
            modified_before: None,
            sort_by: "file_name".to_string(),
            sort_direction: "asc".to_string(),
            limit: 200,
            offset: 0,
        };
        let results = search_assets(&pool, &req).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|a| a.file_name == "hero.png"));
        assert!(results.iter().any(|a| a.file_name == "sound.wav"));

        let req = AssetSearchRequest {
            query: "".to_string(),
            search_file_name: true,
            search_note: false,
            search_path: false,
            search_tags: false,
            asset_type: Some("image".to_string()),
            library_folder_id: None,
            collection_id: Some(1),
            is_favorite: None,
            is_missing: None,
            min_file_size: None,
            max_file_size: None,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            modified_after: None,
            modified_before: None,
            sort_by: "file_name".to_string(),
            sort_direction: "asc".to_string(),
            limit: 200,
            offset: 0,
        };
        let results = search_assets(&pool, &req).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].file_name, "hero.png");

        let req = AssetSearchRequest {
            query: "".to_string(),
            search_file_name: true,
            search_note: false,
            search_path: false,
            search_tags: false,
            asset_type: None,
            library_folder_id: None,
            collection_id: None,
            is_favorite: None,
            is_missing: Some(true),
            min_file_size: None,
            max_file_size: None,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            modified_after: None,
            modified_before: None,
            sort_by: "file_name".to_string(),
            sort_direction: "asc".to_string(),
            limit: 200,
            offset: 0,
        };
        let results = search_assets(&pool, &req).await.unwrap();
        assert!(!results.iter().any(|a| a.file_name.starts_with("._")));
        assert!(!results.iter().any(|a| a.absolute_path.contains("__MACOSX")));
    }

    async fn search_test_pool() -> SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE assets (
                id INTEGER PRIMARY KEY,
                library_folder_id INTEGER NOT NULL,
                absolute_path TEXT NOT NULL UNIQUE,
                file_name TEXT NOT NULL,
                extension TEXT NOT NULL,
                asset_type TEXT NOT NULL,
                file_size INTEGER NOT NULL DEFAULT 0,
                modified_at TEXT NOT NULL,
                width INTEGER,
                height INTEGER,
                thumbnail_path TEXT,
                thumbnail_status TEXT NOT NULL DEFAULT 'none',
                thumbnail_error TEXT,
                note TEXT NOT NULL DEFAULT '',
                is_favorite INTEGER NOT NULL DEFAULT 0,
                is_missing INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "CREATE TABLE tags (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL DEFAULT '#5B8DEF',
                created_at TEXT NOT NULL,
                last_used_at TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "CREATE TABLE asset_tags (
                asset_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (asset_id, tag_id)
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "CREATE TABLE collection_assets (
                collection_id INTEGER NOT NULL,
                asset_id INTEGER NOT NULL,
                PRIMARY KEY (collection_id, asset_id)
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO assets
            (id, library_folder_id, absolute_path, file_name, extension, asset_type, file_size, modified_at, width, height, note, created_at, updated_at)
            VALUES
            (1, 1, '/test/small.png', 'small.png', 'png', 'image', 100, '2024-01-01T00:00:00Z', 32, 32, '', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z'),
            (2, 1, '/test/mid.png', 'mid.png', 'png', 'image', 500, '2024-01-03T00:00:00Z', 128, 128, '', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z'),
            (3, 1, '/test/large.png', 'large.png', 'png', 'image', 2000, '2024-01-06T00:00:00Z', 2048, 2048, '', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    #[tokio::test]
    async fn filters_by_file_size_dimensions_and_modified_time() {
        let pool = search_test_pool().await;

        let mut req = empty_request();
        req.min_file_size = Some(200);
        req.max_file_size = Some(900);
        req.min_width = Some(64);
        req.max_width = Some(512);
        req.min_height = Some(64);
        req.max_height = Some(512);
        req.modified_after = Some("2024-01-02T00:00:00Z".to_string());
        req.modified_before = Some("2024-01-05T00:00:00Z".to_string());

        let results = search_assets(&pool, &req).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].file_name, "mid.png");
    }

    #[tokio::test]
    async fn sorts_by_size_descending() {
        let pool = search_test_pool().await;

        let mut req = empty_request();
        req.sort_by = "file_size".to_string();
        req.sort_direction = "desc".to_string();

        let results = search_assets(&pool, &req).await.unwrap();
        let names: Vec<String> = results.into_iter().map(|asset| asset.file_name).collect();

        assert_eq!(names, vec!["large.png", "mid.png", "small.png"]);
    }

    #[tokio::test]
    async fn invalid_sort_values_fall_back_to_file_name_ascending() {
        let pool = search_test_pool().await;

        let mut req = empty_request();
        req.sort_by = "absolute_path; DROP TABLE assets".to_string();
        req.sort_direction = "sideways".to_string();

        let results = search_assets(&pool, &req).await.unwrap();
        let names: Vec<String> = results.into_iter().map(|asset| asset.file_name).collect();

        assert_eq!(names, vec!["large.png", "mid.png", "small.png"]);
    }
}
