use crate::{
    commands::CommandError,
    db,
    models::{Asset, AssetSearchRequest, FacetTag},
    search,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use tauri::State;

#[derive(Debug, Serialize)]
pub struct IndexedDirectory {
    pub path: String,
    pub count: i64,
}

#[tauri::command]
pub async fn list_indexed_directories(
    db: State<'_, SqlitePool>,
    folder_id: i64,
) -> Result<Vec<IndexedDirectory>, CommandError> {
    directories(&db, folder_id).await.map_err(Into::into)
}
pub async fn directories(
    pool: &SqlitePool,
    folder_id: i64,
) -> anyhow::Result<Vec<IndexedDirectory>> {
    let folder = db::get_folder_by_id(pool, folder_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("资源库不存在"))?;
    let root = folder
        .path
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_string();
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT parent_path, COUNT(*) FROM assets WHERE library_folder_id=? GROUP BY parent_path",
    )
    .bind(folder_id)
    .fetch_all(pool)
    .await?;
    let mut counts = std::collections::BTreeMap::<String, i64>::new();
    for (mut path, count) in rows {
        while path.len() > root.len()
            && path
                .to_lowercase()
                .starts_with(&(root.to_lowercase() + "/"))
        {
            *counts.entry(path.clone()).or_default() += count;
            let Some((parent, _)) = path.rsplit_once('/') else {
                break;
            };
            path = parent.to_string();
        }
    }
    Ok(counts
        .into_iter()
        .map(|(path, count)| IndexedDirectory { path, count })
        .collect())
}

#[tauri::command]
pub async fn list_facet_tags(
    db: State<'_, SqlitePool>,
    request: AssetSearchRequest,
) -> Result<Vec<FacetTag>, CommandError> {
    search::facet_tags(&db, &request).await.map_err(Into::into)
}

pub async fn save_category(
    pool: &SqlitePool,
    name: &str,
    dimension: &str,
    asset_ids: &[i64],
) -> anyhow::Result<()> {
    anyhow::ensure!(
        ["usage", "subject", "action", "style", "general"].contains(&dimension),
        "无效分类维度"
    );
    let name = crate::tags::normalize_tag_name(name);
    anyhow::ensure!(
        !name.is_empty() && name.chars().count() <= 80,
        "标签名称需为 1–80 字"
    );
    anyhow::ensure!(asset_ids.len() <= 2000, "每批最多 2000 个资源");
    let mut tx = pool.begin().await?;
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT OR IGNORE INTO tags(name,color,created_at,last_used_at) VALUES (?,'#5B8DEF',?,?)",
    )
    .bind(&name)
    .bind(&now)
    .bind(&now)
    .execute(&mut *tx)
    .await?;
    let id: i64 = sqlx::query_scalar("SELECT id FROM tags WHERE name=?")
        .bind(&name)
        .fetch_one(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO tag_dimensions(tag_id,dimension) VALUES (?,?) ON CONFLICT(tag_id) DO UPDATE SET dimension=excluded.dimension").bind(id).bind(dimension).execute(&mut *tx).await?;
    for asset_id in asset_ids {
        sqlx::query("INSERT OR IGNORE INTO asset_tags(asset_id,tag_id) VALUES (?,?)")
            .bind(asset_id)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        db::refresh_asset_search_document(&mut tx, *asset_id).await?;
    }
    tx.commit().await?;
    Ok(())
}
#[tauri::command]
pub async fn save_category_tag(
    db: State<'_, SqlitePool>,
    name: String,
    dimension: String,
    asset_ids: Vec<i64>,
) -> Result<(), CommandError> {
    save_category(&db, &name, &dimension, &asset_ids)
        .await
        .map_err(Into::into)
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq, Eq)]
pub struct ClassificationRule {
    pub id: i64,
    pub name: String,
    pub field: String,
    pub pattern: String,
    pub tag_id: i64,
}
#[derive(Debug, Serialize)]
pub struct RuleCandidate {
    pub asset: Asset,
    pub reason: String,
}
#[derive(Debug, Serialize, FromRow)]
pub struct ClassificationBatch {
    pub id: i64,
    pub rule_name: String,
    pub created_at: String,
    pub added_count: i64,
    pub undone: bool,
}

pub fn match_reason(rule: &ClassificationRule, asset: &Asset) -> Option<String> {
    let pattern = rule.pattern.to_lowercase();
    let found = if rule.field == "directory" {
        let path = asset.absolute_path.replace('\\', "/");
        path.rsplit_once('/')
            .map(|(parent, _)| {
                parent
                    .split('/')
                    .any(|segment| segment.to_lowercase() == pattern)
            })
            .unwrap_or(false)
    } else {
        let stem = asset
            .file_name
            .rsplit_once('.')
            .map(|(stem, _)| stem)
            .unwrap_or(&asset.file_name);
        stem.split(|c: char| !c.is_alphanumeric())
            .any(|token| token.to_lowercase() == pattern)
    };
    found.then(|| {
        format!(
            "{}完全匹配「{}」",
            if rule.field == "directory" {
                "目录段"
            } else {
                "文件名词元"
            },
            rule.pattern
        )
    })
}

#[tauri::command]
pub async fn list_classification_rules(
    db: State<'_, SqlitePool>,
) -> Result<Vec<ClassificationRule>, CommandError> {
    Ok(
        sqlx::query_as("SELECT id,name,field,pattern,tag_id FROM classification_rules ORDER BY id")
            .fetch_all(&*db)
            .await?,
    )
}
#[tauri::command]
pub async fn save_classification_rule(
    db: State<'_, SqlitePool>,
    rule: ClassificationRule,
) -> Result<(), CommandError> {
    save_rule(&db, rule).await.map_err(Into::into)
}
pub async fn save_rule(pool: &SqlitePool, rule: ClassificationRule) -> anyhow::Result<()> {
    anyhow::ensure!(
        ["directory", "filename"].contains(&rule.field.as_str()),
        "无效规则字段"
    );
    let pattern = rule.pattern.trim();
    anyhow::ensure!(
        !rule.name.trim().is_empty()
            && rule.name.chars().count() <= 80
            && !pattern.is_empty()
            && pattern.chars().count() <= 80,
        "规则名称和匹配词需为 1–80 字"
    );
    anyhow::ensure!(
        !pattern.contains(['/', '\\'])
            && (rule.field != "filename" || pattern.chars().all(char::is_alphanumeric)),
        "目录规则匹配单个目录段；文件名规则匹配一个独立词元"
    );
    if rule.id == 0 {
        sqlx::query("INSERT INTO classification_rules(name,field,pattern,tag_id) VALUES (?,?,?,?)")
            .bind(rule.name.trim())
            .bind(rule.field)
            .bind(pattern)
            .bind(rule.tag_id)
            .execute(pool)
            .await?;
    } else {
        let result = sqlx::query(
            "UPDATE classification_rules SET name=?,field=?,pattern=?,tag_id=? WHERE id=?",
        )
        .bind(rule.name.trim())
        .bind(rule.field)
        .bind(pattern)
        .bind(rule.tag_id)
        .bind(rule.id)
        .execute(pool)
        .await?;
        anyhow::ensure!(result.rows_affected() == 1, "规则不存在");
    }
    Ok(())
}

#[tauri::command]
pub async fn preview_classification_rule(
    db: State<'_, SqlitePool>,
    rule: ClassificationRule,
    request: AssetSearchRequest,
) -> Result<Vec<RuleCandidate>, CommandError> {
    preview(&db, &rule, request).await.map_err(Into::into)
}
pub async fn preview(
    pool: &SqlitePool,
    rule: &ClassificationRule,
    mut request: AssetSearchRequest,
) -> anyhow::Result<Vec<RuleCandidate>> {
    let current: ClassificationRule =
        sqlx::query_as("SELECT id,name,field,pattern,tag_id FROM classification_rules WHERE id=?")
            .bind(rule.id)
            .fetch_one(pool)
            .await?;
    anyhow::ensure!(&current == rule, "规则已变更，请重新载入");
    request.offset = 0;
    request.limit = 2000;
    let page = search::search_assets_page(pool, &request).await?;
    anyhow::ensure!(
        page.total_count <= 2000,
        "当前范围超过 2000 个资源，请先缩小目录或筛选范围再预览"
    );
    let blocked: Vec<i64> = sqlx::query_scalar("SELECT asset_id FROM asset_tags WHERE tag_id=? UNION SELECT asset_id FROM classification_exclusions WHERE tag_id=?").bind(rule.tag_id).bind(rule.tag_id).fetch_all(pool).await?;
    let blocked: std::collections::HashSet<i64> = blocked.into_iter().collect();
    Ok(page
        .assets
        .into_iter()
        .filter(|a| !a.is_missing && !blocked.contains(&a.id))
        .filter_map(|asset| {
            match_reason(rule, &asset).map(|reason| RuleCandidate { asset, reason })
        })
        .collect())
}

#[tauri::command]
pub async fn apply_classification_rule(
    db: State<'_, SqlitePool>,
    rule: ClassificationRule,
    asset_ids: Vec<i64>,
) -> Result<i64, CommandError> {
    apply(&db, &rule, &asset_ids).await.map_err(Into::into)
}
pub async fn apply(
    pool: &SqlitePool,
    rule: &ClassificationRule,
    ids: &[i64],
) -> anyhow::Result<i64> {
    anyhow::ensure!(
        !ids.is_empty() && ids.len() <= 2000,
        "请勾选 1–2000 个预览资源"
    );
    let mut tx = pool.begin().await?;
    let current: ClassificationRule =
        sqlx::query_as("SELECT id,name,field,pattern,tag_id FROM classification_rules WHERE id=?")
            .bind(rule.id)
            .fetch_one(&mut *tx)
            .await?;
    anyhow::ensure!(&current == rule, "规则已变更，请重新预览");
    let batch =
        sqlx::query("INSERT INTO classification_batches(rule_name,created_at) VALUES (?,?)")
            .bind(&rule.name)
            .bind(chrono::Utc::now().to_rfc3339())
            .execute(&mut *tx)
            .await?
            .last_insert_rowid();
    let mut added = 0;
    for id in ids {
        let asset: Option<Asset> = sqlx::query_as("SELECT * FROM assets WHERE id=?")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
        let Some(asset) = asset else {
            continue;
        };
        if asset.is_missing || match_reason(rule, &asset).is_none() {
            continue;
        }
        let exists: i64 = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM asset_tags WHERE asset_id=?1 AND tag_id=?2) OR EXISTS(SELECT 1 FROM classification_exclusions WHERE asset_id=?1 AND tag_id=?2)").bind(id).bind(rule.tag_id).fetch_one(&mut *tx).await?;
        if exists != 0 {
            continue;
        }
        sqlx::query("INSERT INTO asset_tags(asset_id,tag_id) VALUES (?,?)")
            .bind(id)
            .bind(rule.tag_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("INSERT INTO classification_links(asset_id,tag_id,batch_id) VALUES (?,?,?)")
            .bind(id)
            .bind(rule.tag_id)
            .bind(batch)
            .execute(&mut *tx)
            .await?;
        db::refresh_asset_search_document(&mut tx, *id).await?;
        added += 1;
    }
    sqlx::query("UPDATE classification_batches SET added_count=? WHERE id=?")
        .bind(added)
        .bind(batch)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(added)
}

#[tauri::command]
pub async fn list_classification_batches(
    db: State<'_, SqlitePool>,
) -> Result<Vec<ClassificationBatch>, CommandError> {
    Ok(sqlx::query_as("SELECT id,rule_name,created_at,added_count,undone FROM classification_batches ORDER BY id DESC LIMIT 30").fetch_all(&*db).await?)
}
#[tauri::command]
pub async fn undo_classification_batch(
    db: State<'_, SqlitePool>,
    batch_id: i64,
) -> Result<i64, CommandError> {
    undo(&db, batch_id).await.map_err(Into::into)
}
pub async fn undo(pool: &SqlitePool, batch_id: i64) -> anyhow::Result<i64> {
    let mut tx = pool.begin().await?;
    let links: Vec<(i64, i64)> =
        sqlx::query_as("SELECT asset_id,tag_id FROM classification_links WHERE batch_id=?")
            .bind(batch_id)
            .fetch_all(&mut *tx)
            .await?;
    for (asset, tag) in &links {
        sqlx::query("DELETE FROM asset_tags WHERE asset_id=? AND tag_id=?")
            .bind(asset)
            .bind(tag)
            .execute(&mut *tx)
            .await?;
        // Undo is not a manual rejection; re-applying the rule remains possible.
        sqlx::query("DELETE FROM classification_exclusions WHERE asset_id=? AND tag_id=?")
            .bind(asset)
            .bind(tag)
            .execute(&mut *tx)
            .await?;
        db::refresh_asset_search_document(&mut tx, *asset).await?;
    }
    sqlx::query("UPDATE classification_batches SET undone=1 WHERE id=?")
        .bind(batch_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(links.len() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    async fn fixture() -> (SqlitePool, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let pool = db::connect(&dir.path().join("test.sqlite")).await.unwrap();
        let folder = db::create_library_folder(&pool, "fixture", "C:/assets")
            .await
            .unwrap();
        for (i, path) in [
            "C:/assets/Archer/Hero_Idle.png",
            "C:/assets/Archer/Blue/Hero_Shoot.png",
            "C:/assets/ArcherExtra/Rainbow.png",
            "C:/assets/100%/Bow.png",
        ]
        .iter()
        .enumerate()
        {
            sqlx::query("INSERT INTO assets(id,library_folder_id,absolute_path,file_name,extension,asset_type,file_size,modified_at,created_at,updated_at) VALUES (?,?,?,?,'png','image',10,'','','')")
                .bind(i as i64 + 1).bind(folder.id).bind(path).bind(path.rsplit('/').next().unwrap()).execute(&pool).await.unwrap();
            db::refresh_asset_search_document(&mut pool.acquire().await.unwrap(), i as i64 + 1)
                .await
                .unwrap();
        }
        (pool, dir)
    }
    fn request() -> AssetSearchRequest {
        AssetSearchRequest {
            limit: 200,
            search_file_name: true,
            ..Default::default()
        }
    }
    async fn ids(pool: &SqlitePool, req: &AssetSearchRequest) -> Vec<i64> {
        let mut ids: Vec<_> = search::search_assets(pool, req)
            .await
            .unwrap()
            .iter()
            .map(|a| a.id)
            .collect();
        ids.sort();
        ids
    }
    async fn tag(pool: &SqlitePool, name: &str) -> i64 {
        sqlx::query_scalar("SELECT id FROM tags WHERE name=?")
            .bind(name)
            .fetch_one(pool)
            .await
            .unwrap()
    }
    async fn rule(pool: &SqlitePool, tag_id: i64) -> ClassificationRule {
        save_rule(
            pool,
            ClassificationRule {
                id: 0,
                name: "Archer 分类".into(),
                field: "directory".into(),
                pattern: "Archer".into(),
                tag_id,
            },
        )
        .await
        .unwrap();
        sqlx::query_as("SELECT * FROM classification_rules ORDER BY id DESC LIMIT 1")
            .fetch_one(pool)
            .await
            .unwrap()
    }
    async fn latest_batch(pool: &SqlitePool) -> i64 {
        sqlx::query_scalar("SELECT MAX(id) FROM classification_batches")
            .fetch_one(pool)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn directory_boundaries_recursive_direct_and_literal_wildcards() {
        let (pool, _dir) = fixture().await;
        let mut req = request();
        req.discovery.directory_path = Some("c:\\assets\\Archer\\".into());
        assert_eq!(ids(&pool, &req).await, vec![1, 2]);
        req.discovery.recursive = false;
        assert_eq!(ids(&pool, &req).await, vec![1]);
        req.discovery.directory_path = Some("C:/assets/100%".into());
        req.discovery.recursive = true;
        assert_eq!(ids(&pool, &req).await, vec![4]);
        let dirs = directories(&pool, 1).await.unwrap();
        assert_eq!(
            dirs.iter()
                .find(|d| d.path == "C:/assets/Archer")
                .unwrap()
                .count,
            2
        );
        pool.close().await;
    }
    #[tokio::test]
    async fn facets_are_exact_or_within_group_and_across_groups_and_count_current_results() {
        let (pool, _dir) = fixture().await;
        save_category(&pool, "角色", "usage", &[1]).await.unwrap();
        save_category(&pool, "图标", "usage", &[2, 3])
            .await
            .unwrap();
        save_category(&pool, "弓箭手", "subject", &[1, 2])
            .await
            .unwrap();
        save_category(&pool, "角色参考", "general", &[4])
            .await
            .unwrap();
        let role = tag(&pool, "角色").await;
        let icon = tag(&pool, "图标").await;
        let archer = tag(&pool, "弓箭手").await;
        let mut req = request();
        req.discovery.tag_ids = vec![role, icon, archer];
        assert_eq!(ids(&pool, &req).await, vec![1, 2]);
        req.discovery.excluded_tag_ids = vec![icon];
        assert_eq!(ids(&pool, &req).await, vec![1]);
        let facets = search::facet_tags(&pool, &req).await.unwrap();
        assert_eq!(facets.iter().find(|t| t.id == role).unwrap().asset_count, 1);
        assert_eq!(facets.iter().find(|t| t.id == icon).unwrap().asset_count, 0);
        req = request();
        req.discovery.unclassified = true;
        assert_eq!(ids(&pool, &req).await, vec![4]);
        pool.close().await;
    }
    #[tokio::test]
    async fn preview_apply_undo_preserve_existing_manual_links_and_refresh_search() {
        let (pool, _dir) = fixture().await;
        save_category(&pool, "角色", "usage", &[1]).await.unwrap();
        let rule = rule(&pool, tag(&pool, "角色").await).await;
        let candidates = preview(&pool, &rule, request()).await.unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].asset.id, 2);
        assert_eq!(apply(&pool, &rule, &[1, 2, 3]).await.unwrap(), 1);
        let batch = latest_batch(&pool).await;
        assert_eq!(apply(&pool, &rule, &[2]).await.unwrap(), 0);
        let mut req = request();
        req.search_tags = true;
        req.query = "角色".into();
        assert_eq!(ids(&pool, &req).await, vec![1, 2]);
        assert_eq!(undo(&pool, batch).await.unwrap(), 1);
        assert_eq!(ids(&pool, &req).await, vec![1]);
        assert_eq!(undo(&pool, batch).await.unwrap(), 0);
        assert_eq!(preview(&pool, &rule, request()).await.unwrap().len(), 1);
        pool.close().await;
    }
    #[tokio::test]
    async fn manual_takeover_removal_and_rename_survive_rule_reapply_and_undo() {
        let (pool, _dir) = fixture().await;
        save_category(&pool, "角色", "usage", &[]).await.unwrap();
        let id = tag(&pool, "角色").await;
        let rule = rule(&pool, id).await;
        apply(&pool, &rule, &[1, 2]).await.unwrap();
        let batch = latest_batch(&pool).await;
        db::apply_tag_to_assets(&pool, "角色", &[1]).await.unwrap();
        db::remove_tag_from_assets(&pool, id, &[2]).await.unwrap();
        assert_eq!(apply(&pool, &rule, &[2]).await.unwrap(), 0);
        assert_eq!(undo(&pool, batch).await.unwrap(), 0);
        assert_eq!(db::list_asset_tags(&pool, 1).await.unwrap(), vec!["角色"]);
        db::apply_tag_to_assets(&pool, "角色", &[2]).await.unwrap();
        assert!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM classification_exclusions")
                .fetch_one(&pool)
                .await
                .unwrap()
                == 0
        );
        db::remove_tag_from_assets(&pool, id, &[1]).await.unwrap();
        // A new label may be introduced by a rule; semantic rename takes ownership.
        save_category(&pool, "测试", "subject", &[]).await.unwrap();
        let next = rule.clone();
        let mut next = ClassificationRule {
            id: 0,
            tag_id: tag(&pool, "测试").await,
            ..next
        };
        save_rule(&pool, next.clone()).await.unwrap();
        next.id = sqlx::query_scalar("SELECT MAX(id) FROM classification_rules")
            .fetch_one(&pool)
            .await
            .unwrap();
        apply(&pool, &next, &[1]).await.unwrap();
        let batch = latest_batch(&pool).await;
        db::update_tag(&pool, next.tag_id, "人工命名", "#123456")
            .await
            .unwrap();
        assert_eq!(undo(&pool, batch).await.unwrap(), 0);
        assert!(db::list_asset_tags(&pool, 1)
            .await
            .unwrap()
            .contains(&"人工命名".into()));
        db::delete_library_folder(&pool, 1).await.unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM classification_links")
                .fetch_one(&pool)
                .await
                .unwrap(),
            0
        );
        pool.close().await;
    }
    #[tokio::test]
    async fn stale_rules_are_rejected_and_tokens_do_not_match_substrings() {
        let (pool, _dir) = fixture().await;
        save_category(&pool, "弓", "subject", &[]).await.unwrap();
        let mut r = rule(&pool, tag(&pool, "弓").await).await;
        r.field = "filename".into();
        r.pattern = "Bow".into();
        save_rule(&pool, r.clone()).await.unwrap();
        let items = preview(&pool, &r, request()).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].asset.id, 4);
        let old = r.clone();
        r.pattern = "Idle".into();
        save_rule(&pool, r.clone()).await.unwrap();
        assert!(apply(&pool, &old, &[4]).await.is_err());
        assert!(preview(&pool, &old, request()).await.is_err());
        pool.close().await;
    }
}
