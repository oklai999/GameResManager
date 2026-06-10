use sqlx::SqlitePool;
use crate::models::{Asset, AssetSearchRequest};

#[allow(dead_code)]
fn escape_like_pattern(input: &str) -> String {
    input.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
}

fn build_fts_query(req: &AssetSearchRequest) -> Option<String> {
    if req.query.is_empty() {
        return None;
    }

    let terms: Vec<String> = req.query
        .split_whitespace()
        .map(|t| t.replace('"', "\"\""))
        .filter(|t| !t.is_empty())
        .collect();

    if terms.is_empty() {
        return None;
    }

    let mut columns: Vec<&str> = Vec::new();
    if req.search_file_name { columns.push("file_name"); }
    if req.search_note { columns.push("note"); }
    if req.search_path { columns.push("absolute_path"); }
    if req.search_tags { columns.push("tags"); }

    if columns.is_empty() {
        return Some(String::new());
    }

    let term_exprs: Vec<String> = terms.iter().map(|term| {
        if columns.len() == 4 {
            format!("\"{}\"*", term)
        } else {
            let col_exprs: Vec<String> = columns.iter()
                .map(|col| format!("{}:\"{}\"*", col, term))
                .collect();
            if col_exprs.len() == 1 {
                col_exprs.into_iter().next().unwrap()
            } else {
                format!("({})", col_exprs.join(" OR "))
            }
        }
    }).collect();

    Some(term_exprs.join(" AND "))
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
    conditions.push(exclude_system_metadata_condition());

    let fts_query = build_fts_query(req);
    let has_fts = fts_query.as_ref().map_or(false, |q| !q.is_empty());
    let force_zero = fts_query.as_ref().map_or(false, |q| q.is_empty());

    if force_zero {
        conditions.push("1 = 0".to_string());
    } else if has_fts {
        conditions.push("assets.id IN (SELECT rowid FROM asset_search_fts WHERE asset_search_fts MATCH ?)".to_string());
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

    if has_fts {
        if let Some(ref fts) = fts_query {
            query = query.bind(fts);
        }
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

pub async fn count_search_assets(db: &SqlitePool, req: &AssetSearchRequest) -> anyhow::Result<i64> {
    let mut sql = String::from("SELECT COUNT(*) FROM assets");

    let mut conditions: Vec<String> = Vec::new();
    conditions.push(exclude_system_metadata_condition());

    let fts_query = build_fts_query(req);
    let has_fts = fts_query.as_ref().map_or(false, |q| !q.is_empty());
    let force_zero = fts_query.as_ref().map_or(false, |q| q.is_empty());

    if force_zero {
        conditions.push("1 = 0".to_string());
    } else if has_fts {
        conditions.push("assets.id IN (SELECT rowid FROM asset_search_fts WHERE asset_search_fts MATCH ?)".to_string());
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

    let mut query = sqlx::query_scalar::<_, i64>(&sql);

    if has_fts {
        if let Some(ref fts) = fts_query {
            query = query.bind(fts);
        }
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

    let count = query.fetch_one(db).await?;
    Ok(count)
}

pub async fn search_assets_page(
    db: &SqlitePool,
    req: &AssetSearchRequest,
) -> anyhow::Result<crate::models::AssetSearchResponse> {
    let assets = search_assets(db, req).await?;
    let total_count = count_search_assets(db, req).await?;
    let limit = req.limit.clamp(1, 2000);
    let offset = req.offset.max(0);
    Ok(crate::models::AssetSearchResponse {
        assets,
        total_count,
        limit,
        offset,
    })
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
    fn build_fts_query_returns_none_for_empty_query() {
        let req = empty_request();
        assert_eq!(build_fts_query(&req), None);
    }

    #[test]
    fn build_fts_query_returns_empty_string_when_no_scopes() {
        let mut req = empty_request();
        req.query = "hero".to_string();
        req.search_file_name = false;
        assert_eq!(build_fts_query(&req), Some(String::new()));
    }

    #[test]
    fn build_fts_query_splits_terms_and_joins_with_and() {
        let mut req = empty_request();
        req.query = "hero idle".to_string();
        req.search_file_name = true;
        req.search_note = true;
        req.search_path = true;
        req.search_tags = true;
        assert_eq!(build_fts_query(&req), Some("\"hero\"* AND \"idle\"*".to_string()));
    }

    #[test]
    fn build_fts_query_escapes_double_quotes() {
        let mut req = empty_request();
        req.query = "5\" sword".to_string();
        req.search_file_name = true;
        req.search_note = true;
        req.search_path = true;
        req.search_tags = true;
        assert_eq!(build_fts_query(&req), Some("\"5\"\"\"* AND \"sword\"*".to_string()));
    }

    #[test]
    fn build_fts_query_uses_prefix_match_for_all_columns() {
        let mut req = empty_request();
        req.query = "hero".to_string();
        req.search_file_name = true;
        req.search_note = true;
        req.search_path = true;
        req.search_tags = true;
        assert_eq!(build_fts_query(&req), Some("\"hero\"*".to_string()));
    }

    #[test]
    fn build_fts_query_uses_column_scoped_prefix_match() {
        let mut req = empty_request();
        req.query = "hero".to_string();
        req.search_file_name = true;
        req.search_note = false;
        req.search_path = false;
        req.search_tags = true;
        assert_eq!(
            build_fts_query(&req),
            Some("(file_name:\"hero\"* OR tags:\"hero\"*)".to_string())
        );
    }

    #[test]
    fn build_fts_query_single_scope_no_parens() {
        let mut req = empty_request();
        req.query = "hero".to_string();
        req.search_file_name = false;
        req.search_note = true;
        req.search_path = false;
        req.search_tags = false;
        assert_eq!(build_fts_query(&req), Some("note:\"hero\"*".to_string()));
    }

    #[test]
    fn build_fts_query_filters_out_empty_terms() {
        let mut req = empty_request();
        req.query = "hero   ".to_string();
        req.search_file_name = true;
        req.search_note = true;
        req.search_path = true;
        req.search_tags = true;
        assert_eq!(build_fts_query(&req), Some("\"hero\"*".to_string()));
    }

    #[test]
    fn build_fts_query_all_whitespace_returns_none() {
        let mut req = empty_request();
        req.query = "    ".to_string();
        req.search_file_name = true;
        assert_eq!(build_fts_query(&req), None);
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
        sqlx::query(
            "CREATE VIRTUAL TABLE asset_search_fts USING fts5(
                file_name, absolute_path, note, tags, tokenize = 'unicode61'
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
        sqlx::query(
            "INSERT INTO asset_search_fts (rowid, file_name, absolute_path, note, tags) VALUES
            (1, 'hero.png', '/test/hero.png', 'main character', ''),
            (2, 'villain.png', '/test/villain.png', '', 'important'),
            (3, 'sound.wav', '/test/sound.wav', '', ''),
            (4, '._hero.png', '/test/__MACOSX/._hero.png', '', '')"
        ).execute(&pool).await.unwrap();

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

    #[tokio::test]
    async fn paged_search_returns_total_count() {
        let pool = search_test_pool().await;
        let mut req = empty_request();
        req.limit = 2;
        req.offset = 0;

        let page = search_assets_page(&pool, &req).await.unwrap();

        assert_eq!(page.assets.len(), 2);
        assert_eq!(page.total_count, 3);
        assert_eq!(page.limit, 2);
        assert_eq!(page.offset, 0);
    }

    #[tokio::test]
    async fn paged_search_applies_same_filters_to_count() {
        let pool = search_test_pool().await;
        let mut req = empty_request();
        req.min_file_size = Some(200);
        req.max_file_size = Some(900);
        req.limit = 20;
        req.offset = 0;

        let page = search_assets_page(&pool, &req).await.unwrap();

        assert_eq!(page.assets.len(), 1);
        assert_eq!(page.total_count, 1);
        assert_eq!(page.assets[0].file_name, "mid.png");
    }

    #[tokio::test]
    async fn paged_search_uses_fts_for_tag_and_note_text() {
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
            "CREATE VIRTUAL TABLE asset_search_fts USING fts5(
                file_name, absolute_path, note, tags, tokenize = 'unicode61'
            )"
        ).execute(&pool).await.unwrap();

        sqlx::query(
            "INSERT INTO assets (id, library_folder_id, absolute_path, file_name, extension, asset_type, modified_at, note, is_favorite, is_missing, created_at, updated_at) VALUES
            (1, 1, '/test/hero.png', 'hero.png', 'png', 'image', '2024-01-01T00:00:00Z', '主角待机', 0, 0, '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z'),
            (2, 1, '/test/tree.png', 'tree.png', 'png', 'image', '2024-01-01T00:00:00Z', '森林背景', 0, 0, '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "INSERT INTO asset_search_fts (rowid, file_name, absolute_path, note, tags) VALUES
            (1, 'hero.png', '/test/hero.png', '主角待机', '角色'),
            (2, 'tree.png', '/test/tree.png', '森林背景', '场景')"
        ).execute(&pool).await.unwrap();

        // Test complete-token match with Chinese text
        let mut req = empty_request();
        req.query = "主角待机".to_string();
        req.search_note = true;

        let page = search_assets_page(&pool, &req).await.unwrap();
        assert_eq!(page.total_count, 1);
        assert_eq!(page.assets.len(), 1);
        assert_eq!(page.assets[0].file_name, "hero.png");

        // Test tag search with Chinese text
        let mut req = empty_request();
        req.query = "角色".to_string();
        req.search_tags = true;

        let page = search_assets_page(&pool, &req).await.unwrap();
        assert_eq!(page.total_count, 1);
        assert_eq!(page.assets.len(), 1);
        assert_eq!(page.assets[0].file_name, "hero.png");

        // Test combined note + tag search across multiple terms
        let mut req = empty_request();
        req.query = "角色 主角待机".to_string();
        req.search_note = true;
        req.search_tags = true;

        let page = search_assets_page(&pool, &req).await.unwrap();
        assert_eq!(page.total_count, 1);
        assert_eq!(page.assets.len(), 1);
        assert_eq!(page.assets[0].file_name, "hero.png");

        // Verify partial Chinese keyword matches via prefix query
        let mut req = empty_request();
        req.query = "主角".to_string();
        req.search_note = true;

        let page = search_assets_page(&pool, &req).await.unwrap();
        assert_eq!(page.total_count, 1);
        assert_eq!(page.assets[0].file_name, "hero.png");
    }

    async fn scoped_search_pool() -> SqlitePool {
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
            "CREATE VIRTUAL TABLE asset_search_fts USING fts5(
                file_name, absolute_path, note, tags, tokenize = 'unicode61'
            )"
        ).execute(&pool).await.unwrap();

        sqlx::query(
            "INSERT INTO assets (id, library_folder_id, absolute_path, file_name, extension, asset_type, modified_at, note, is_favorite, is_missing, created_at, updated_at) VALUES
            (1, 1, '/test/hero.png', 'hero.png', 'png', 'image', '2024-01-01T00:00:00Z', 'main character', 0, 0, '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z'),
            (2, 1, '/test/villain.png', 'villain.png', 'png', 'image', '2024-01-01T00:00:00Z', '', 0, 0, '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z'),
            (3, 1, '/test/sound.wav', 'sound.wav', 'wav', 'audio', '2024-01-01T00:00:00Z', '', 0, 0, '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')"
        ).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO tags (id, name, color, created_at, last_used_at) VALUES (1, 'important', '#FF0000', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO asset_tags (asset_id, tag_id) VALUES (2, 1)")
            .execute(&pool).await.unwrap();
        sqlx::query(
            "INSERT INTO asset_search_fts (rowid, file_name, absolute_path, note, tags) VALUES
            (1, 'hero.png', '/test/hero.png', 'main character', ''),
            (2, 'villain.png', '/test/villain.png', '', 'important'),
            (3, 'sound.wav', '/test/sound.wav', '', '')"
        ).execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn paged_search_with_no_scopes_returns_zero() {
        let pool = scoped_search_pool().await;
        let mut req = empty_request();
        req.query = "hero".to_string();
        req.search_file_name = false;
        req.search_note = false;
        req.search_path = false;
        req.search_tags = false;

        let page = search_assets_page(&pool, &req).await.unwrap();
        assert_eq!(page.total_count, 0);
        assert!(page.assets.is_empty());
    }

    #[tokio::test]
    async fn paged_search_isolates_file_name_scope() {
        let pool = scoped_search_pool().await;
        let mut req = empty_request();
        req.query = "hero".to_string();
        req.search_file_name = true;

        let page = search_assets_page(&pool, &req).await.unwrap();
        assert_eq!(page.total_count, 1);
        assert_eq!(page.assets[0].file_name, "hero.png");
    }

    #[tokio::test]
    async fn paged_search_isolates_note_scope() {
        let pool = scoped_search_pool().await;
        let mut req = empty_request();
        req.query = "main".to_string();
        req.search_file_name = false;
        req.search_note = true;

        let page = search_assets_page(&pool, &req).await.unwrap();
        assert_eq!(page.total_count, 1);
        assert_eq!(page.assets[0].file_name, "hero.png");
    }

    #[tokio::test]
    async fn paged_search_isolates_path_scope() {
        let pool = scoped_search_pool().await;
        let mut req = empty_request();
        req.query = "villain".to_string();
        req.search_file_name = false;
        req.search_path = true;

        let page = search_assets_page(&pool, &req).await.unwrap();
        assert_eq!(page.total_count, 1);
        assert_eq!(page.assets[0].file_name, "villain.png");
    }

    #[tokio::test]
    async fn paged_search_isolates_tags_scope() {
        let pool = scoped_search_pool().await;
        let mut req = empty_request();
        req.query = "important".to_string();
        req.search_file_name = false;
        req.search_tags = true;

        let page = search_assets_page(&pool, &req).await.unwrap();
        assert_eq!(page.total_count, 1);
        assert_eq!(page.assets[0].file_name, "villain.png");
    }

    #[tokio::test]
    async fn paged_search_fts_with_other_filters_preserves_row_count_consistency() {
        let pool = scoped_search_pool().await;
        let mut req = empty_request();
        req.query = "png".to_string();
        req.search_file_name = true;
        req.asset_type = Some("image".to_string());
        req.limit = 1;
        req.offset = 0;

        let page = search_assets_page(&pool, &req).await.unwrap();
        assert_eq!(page.assets.len(), 1);
        assert_eq!(page.total_count, 2);
        assert!(page.assets.iter().any(|a| a.file_name == "hero.png"));
    }

    #[tokio::test]
    async fn paged_search_handles_risky_input_safely() {
        let pool = scoped_search_pool().await;

        // Double quotes inside term
        let mut req = empty_request();
        req.query = "5\" hero".to_string();
        req.search_file_name = true;
        let page = search_assets_page(&pool, &req).await.unwrap();
        assert_eq!(page.total_count, 0);

        // FTS operators inside quotes should be literal
        let mut req = empty_request();
        req.query = "AND OR NOT".to_string();
        req.search_file_name = true;
        let page = search_assets_page(&pool, &req).await.unwrap();
        assert_eq!(page.total_count, 0);

        // Punctuation should not cause errors
        let mut req = empty_request();
        req.query = "hero.png!@#".to_string();
        req.search_file_name = true;
        let page = search_assets_page(&pool, &req).await.unwrap();
        assert_eq!(page.total_count, 1);
        assert_eq!(page.assets[0].file_name, "hero.png");
    }

    async fn cjk_search_pool() -> SqlitePool {
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
            "CREATE VIRTUAL TABLE asset_search_fts USING fts5(
                file_name, absolute_path, note, tags, tokenize = 'unicode61'
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE VIRTUAL TABLE asset_search_trigram_fts USING fts5(
                file_name, absolute_path, note, tags, tokenize = 'trigram'
            )"
        ).execute(&pool).await.unwrap();

        sqlx::query(
            "INSERT INTO assets (id, library_folder_id, absolute_path, file_name, extension, asset_type, modified_at, note, is_favorite, is_missing, created_at, updated_at) VALUES
            (1, 1, '/test/角色/hero.png', 'hero.png', 'png', 'image', '2024-01-01T00:00:00Z', '主角待机动画', 0, 0, '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z'),
            (2, 1, '/test/场景/tree.png', '森林背景树.png', 'png', 'image', '2024-01-01T00:00:00Z', '白天场景', 0, 0, '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "INSERT INTO tags (id, name, color, created_at, last_used_at) VALUES
            (1, '角色', '#5B8DEF', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z'),
            (2, '动画', '#5B8DEF', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z'),
            (3, '场景', '#5B8DEF', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z'),
            (4, '背景', '#5B8DEF', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "INSERT INTO asset_tags (asset_id, tag_id) VALUES (1, 1), (1, 2), (2, 3), (2, 4)"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "INSERT INTO asset_search_fts (rowid, file_name, absolute_path, note, tags) VALUES
            (1, 'hero.png', '/test/角色/hero.png', '主角待机动画', '角色 动画'),
            (2, '森林背景树.png', '/test/场景/tree.png', '白天场景', '场景 背景')"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "INSERT INTO asset_search_trigram_fts (rowid, file_name, absolute_path, note, tags) VALUES
            (1, 'hero.png', '/test/角色/hero.png', '主角待机动画', '角色 动画'),
            (2, '森林背景树.png', '/test/场景/tree.png', '白天场景', '场景 背景')"
        ).execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn cjk_substring_matches_middle_of_note() {
        let pool = cjk_search_pool().await;
        let mut req = empty_request();
        req.query = "待机".to_string();
        req.search_file_name = false;
        req.search_note = true;

        let page = search_assets_page(&pool, &req).await.unwrap();

        assert_eq!(page.total_count, 1);
        assert_eq!(page.assets[0].file_name, "hero.png");
    }

    #[tokio::test]
    async fn cjk_substring_matches_middle_of_file_name() {
        let pool = cjk_search_pool().await;
        let mut req = empty_request();
        req.query = "背景树".to_string();
        req.search_file_name = true;

        let page = search_assets_page(&pool, &req).await.unwrap();

        assert_eq!(page.total_count, 1);
        assert_eq!(page.assets[0].file_name, "森林背景树.png");
    }

    #[tokio::test]
    async fn cjk_substring_respects_selected_scope() {
        let pool = cjk_search_pool().await;
        let mut req = empty_request();
        req.query = "角色".to_string();
        req.search_file_name = true;
        req.search_path = false;
        req.search_note = false;
        req.search_tags = false;

        let page = search_assets_page(&pool, &req).await.unwrap();

        assert_eq!(page.total_count, 0);
    }

    #[tokio::test]
    async fn multiple_substring_terms_use_and_semantics() {
        let pool = cjk_search_pool().await;
        let mut req = empty_request();
        req.query = "主角 动画".to_string();
        req.search_file_name = false;
        req.search_note = true;
        req.search_tags = true;

        let page = search_assets_page(&pool, &req).await.unwrap();

        assert_eq!(page.total_count, 1);
        assert_eq!(page.assets[0].file_name, "hero.png");
    }
}
