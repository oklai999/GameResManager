CREATE VIRTUAL TABLE IF NOT EXISTS asset_search_fts USING fts5(
  file_name,
  absolute_path,
  note,
  tags,
  tokenize = 'unicode61'
);

INSERT INTO asset_search_fts (rowid, file_name, absolute_path, note, tags)
SELECT
  a.id,
  a.file_name,
  a.absolute_path,
  a.note,
  COALESCE(
    (
      SELECT group_concat(t.name, ' ')
      FROM (
        SELECT t2.name
        FROM tags t2
        INNER JOIN asset_tags at ON at.tag_id = t2.id
        WHERE at.asset_id = a.id
        ORDER BY t2.name
      ) t
    ),
    ''
  )
FROM assets a;
