import { useRef, useState } from "react";
import { DIMENSIONS, saveCategoryTag, type Dimension, type DiscoveryFilter, type FacetTag } from "../api/discovery";

export function CategoryFilters({ tags, filter, onChange, selectedIds, onSaved, onRules, error }: {
  tags: FacetTag[]; filter: DiscoveryFilter; onChange: (filter: DiscoveryFilter) => void;
  selectedIds: number[]; onSaved: () => Promise<void>; onRules: () => void; error: string;
}) {
  const [name, setName] = useState("");
  const [dimension, setDimension] = useState<Dimension>("usage");
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState("");
  const editorRef = useRef<HTMLDetailsElement>(null);
  function toggle(id: number, excluded: boolean) {
    const key = excluded ? "excluded_tag_ids" : "tag_ids";
    const other = excluded ? "tag_ids" : "excluded_tag_ids";
    onChange({ ...filter, [key]: filter[key].includes(id) ? filter[key].filter(v => v !== id) : [...filter[key], id], [other]: filter[other].filter(v => v !== id) });
  }
  async function save(apply: boolean) {
    setSaving(true); setMessage("");
    try { await saveCategoryTag(name, dimension, apply ? selectedIds : []); await onSaved(); setMessage(apply ? "已分类选中资源" : "标签分组已保存"); }
    catch (e) { setMessage(String((e as Error)?.message ?? e)); } finally { setSaving(false); }
  }
  return <section className="category-filters" aria-label="分类筛选">
    <p className="muted">同组任意匹配，跨组同时满足。数字为当前结果中的资源数。</p>
    <label><input type="checkbox" checked={filter.unclassified} onChange={e => onChange({ ...filter, unclassified: e.target.checked })} />尚未分类（无用途/对象/动作/风格）</label>
    {error && <p role="alert">分类数量加载失败：{error}</p>}
    {(Object.keys(DIMENSIONS) as Dimension[]).map(group => <fieldset key={group}><legend>{DIMENSIONS[group]}</legend>
      {tags.filter(t => t.dimension === group).map(tag => <div className="category-row" key={tag.id}>
        <button aria-pressed={filter.tag_ids.includes(tag.id)} onClick={() => toggle(tag.id, false)}>{tag.name} <small>{tag.asset_count}</small></button>
        <button aria-label={`排除 ${tag.name}`} aria-pressed={filter.excluded_tag_ids.includes(tag.id)} onClick={() => toggle(tag.id, true)}>排除</button>
        <button aria-label={`编辑 ${tag.name} 的分组`} onClick={() => { setName(tag.name); setDimension(tag.dimension); if (editorRef.current) editorRef.current.open = true; }}>分组</button>
      </div>)}
    </fieldset>)}
    <details ref={editorRef}><summary>新建/调整分类与批量应用</summary>
      <p className="muted">同名标签共用一个分组。保存分组会更新已有同名标签。</p>
      <input aria-label="分类标签名称" placeholder="例如：角色、弓箭手" value={name} onChange={e => setName(e.target.value)} />
      <select aria-label="分类维度" value={dimension} onChange={e => setDimension(e.target.value as Dimension)}>{Object.entries(DIMENSIONS).map(([key, label]) => <option key={key} value={key}>{label}</option>)}</select>
      <button disabled={saving || !name.trim()} onClick={() => void save(false)}>保存标签分组</button>
      <button disabled={saving || !name.trim() || !selectedIds.length} onClick={() => void save(true)}>应用到选中 {selectedIds.length} 项</button>
      {message && <p role="status">{message}</p>}
    </details>
    <button onClick={onRules}>本地分类规则…</button>
  </section>;
}
