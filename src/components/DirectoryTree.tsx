import { useEffect, useState } from "react";
import { listIndexedDirectories, type IndexedDirectory } from "../api/discovery";
import type { LibraryFolder } from "../types/asset";

export function DirectoryTree({ folder, selectedPath, onSelect, version }: {
  folder: LibraryFolder; selectedPath: string | null; onSelect: (path: string | null) => void; version: number;
}) {
  const [entries, setEntries] = useState<IndexedDirectory[]>([]);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const [retry, setRetry] = useState(0);
  const root = folder.path.replace(/\\/g, "/").replace(/\/$/, "");
  useEffect(() => {
    let cancelled = false; setLoading(true); setEntries([]); setError("");
    listIndexedDirectories(folder.id).then(rows => { if (!cancelled) setEntries(rows); })
      .catch(e => { if (!cancelled) setError(String(e?.message ?? e)); }).finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [folder.id, version, retry]);
  const children = new Map<string, IndexedDirectory[]>();
  for (const entry of entries) {
    const parent = entry.path.slice(0, entry.path.lastIndexOf("/"));
    const key = parent.toLowerCase(); children.set(key, [...(children.get(key) ?? []), entry]);
  }
  function branch(parent: string, depth: number): React.ReactNode {
    return children.get(parent.toLowerCase())?.map(entry => {
      const hasChildren = children.has(entry.path.toLowerCase());
      const label = entry.path.slice(entry.path.lastIndexOf("/") + 1);
      const button = <button className={selectedPath === entry.path ? "directory-selected" : ""} title={entry.path} onClick={() => onSelect(entry.path)}>{label} <small>{entry.count}</small></button>;
      return hasChildren ? <details key={entry.path} open={selectedPath?.startsWith(entry.path + "/") || undefined}>
        <summary>{button}</summary><div className="directory-children">{depth < 50 ? branch(entry.path, depth + 1) : <span>目录层级过深，请使用路径搜索</span>}</div>
      </details> : <div key={entry.path} className="directory-leaf">{button}</div>;
    });
  }
  return <section className="directory-tree" aria-label="索引目录">
    <div className="panel-heading">子目录 <small>含子目录资源数</small></div>
    <button onClick={() => onSelect(null)}>整个素材包</button>
    {loading && <p role="status">读取目录…</p>}
    {error && <p role="alert">{error}<button onClick={() => setRetry(v => v + 1)}>重试</button></p>}
    {!loading && !error && entries.length === 0 && <p>索引中没有子目录</p>}
    {branch(root, 0)}
  </section>;
}
