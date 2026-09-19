import { useEffect, useRef, useState } from "react";
import { applyClassificationRule, listClassificationBatches, listClassificationRules, previewClassificationRule, saveClassificationRule, undoClassificationBatch, type ClassificationBatch, type ClassificationRule, type FacetTag, type RuleCandidate } from "../api/discovery";
import type { AssetSearchRequest } from "../types/asset";

export function ClassificationRules({ tags, request, onClose, onChanged }: {
  tags: FacetTag[]; request: AssetSearchRequest; onClose: () => void; onChanged: () => Promise<void>;
}) {
  const [rules, setRules] = useState<ClassificationRule[]>([]);
  const [batches, setBatches] = useState<ClassificationBatch[]>([]);
  const blank = (): ClassificationRule => ({ id: 0, name: "", field: "directory", pattern: "", tag_id: tags[0]?.id ?? 0 });
  const [draft, setDraft] = useState<ClassificationRule>(blank);
  const [preview, setPreview] = useState<RuleCandidate[] | null>(null);
  const [checked, setChecked] = useState<number[]>([]);
  const [page, setPage] = useState(0);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");
  const version = useRef(0);
  const dialog = useRef<HTMLElement>(null);
  const requestKey = JSON.stringify(request);
  const saved = rules.find(rule => rule.id === draft.id);
  const dirty = JSON.stringify(saved) !== JSON.stringify(draft);
  async function load() {
    const [nextRules, nextBatches] = await Promise.all([listClassificationRules(), listClassificationBatches()]);
    setRules(nextRules); setBatches(nextBatches);
    return nextRules;
  }
  useEffect(() => {
    let cancelled = false;
    const previous = document.activeElement as HTMLElement | null;
    dialog.current?.focus();
    Promise.all([listClassificationRules(), listClassificationBatches()]).then(([r,b]) => {
      if (!cancelled) { setRules(r); setBatches(b); if (r.length) setDraft(r[0]); }
    }).catch(e => { if (!cancelled) setMessage(String(e?.message ?? e)); });
    return () => { cancelled = true; version.current++; previous?.focus(); };
  }, []);
  useEffect(() => { version.current++; setPreview(null); setChecked([]); setPage(0); }, [requestKey]);
  function edit(next: ClassificationRule) { version.current++; setDraft(next); setPreview(null); setChecked([]); setPage(0); setMessage(""); }
  async function run(action: () => Promise<void>) {
    setBusy(true); setMessage("");
    try { await action(); } catch (e) { setMessage(String((e as Error)?.message ?? e)); } finally { setBusy(false); }
  }
  async function generatePreview() {
    const stamp = ++version.current;
    setPreview(null); setChecked([]); setPage(0);
    await run(async () => {
      const candidates = await previewClassificationRule(draft, request);
      if (version.current === stamp) setPreview(candidates);
    });
  }
  return <div className="classification-backdrop"><section className="classification-dialog" ref={dialog} role="dialog" aria-modal="true" aria-label="本地分类规则" tabIndex={-1}
    onKeyDown={e => {
      if (e.key === "Escape" && !busy) onClose();
      if (e.key === "Tab") {
        const nodes = Array.from(dialog.current?.querySelectorAll<HTMLElement>("button:not(:disabled), input:not(:disabled), select:not(:disabled)") ?? []);
        const first = nodes[0], last = nodes[nodes.length - 1];
        if (e.shiftKey && (document.activeElement === first || document.activeElement === dialog.current)) { e.preventDefault(); last?.focus(); }
        else if (!e.shiftKey && (document.activeElement === last || document.activeElement === dialog.current)) { e.preventDefault(); first?.focus(); }
      }
    }}>
    <header><h2>本地分类规则</h2><button disabled={busy} onClick={onClose}>关闭规则</button></header>
    <p>规则只提出标签建议，先预览并勾选后应用。人工移除的标签不会被规则重新添加。</p>
    <fieldset disabled={busy} className="rule-editor">
      <label>已有规则<select aria-label="已有分类规则" value={draft.id} onChange={e => edit(rules.find(r => r.id === Number(e.target.value)) ?? blank())}><option value={0}>新建规则</option>{rules.map(r => <option key={r.id} value={r.id}>{r.name}</option>)}</select></label>
      <label>规则名称<input aria-label="规则名称" value={draft.name} onChange={e => edit({ ...draft, name: e.target.value })} /></label>
      <label>匹配位置<select aria-label="规则匹配位置" value={draft.field} onChange={e => edit({ ...draft, field: e.target.value as ClassificationRule["field"] })}><option value="directory">完整目录段</option><option value="filename">文件名独立词元</option></select></label>
      <label>匹配词<input aria-label="规则匹配词" placeholder={draft.field === "directory" ? "例如 Archer、Units" : "例如 Idle、Shoot"} value={draft.pattern} onChange={e => edit({ ...draft, pattern: e.target.value })} /></label>
      <label>建议标签<select aria-label="规则目标标签" value={draft.tag_id} onChange={e => edit({ ...draft, tag_id: Number(e.target.value) })}><option value={0}>请选择标签</option>{tags.map(t => <option key={t.id} value={t.id}>{t.name}</option>)}</select></label>
      <button disabled={!draft.name.trim() || !draft.pattern.trim() || !draft.tag_id} onClick={() => void run(async () => {
        await saveClassificationRule(draft); const r = await load(); edit(r.find(r => r.id === draft.id) ?? r[r.length - 1]); setMessage("规则已保存，可预览当前查找范围");
      })}>保存规则</button>
      <button disabled={!saved || dirty} onClick={() => void generatePreview()}>预览当前查找范围</button>
    </fieldset>
    <p className="muted">词元以空格、下划线、连字符等分隔，忽略大小写。例如 Bow 匹配 Archer_Bow，但不匹配 Rainbow。目录匹配完整一层。预览范围最多 2000 个资源，超出时请先缩小范围。</p>
    {message && <p role="status">{message}</p>}
    {busy && <p role="status">处理中…</p>}
    {preview !== null && <section aria-label="分类建议预览">
      <div className="rule-actions"><strong>{preview.length} 项新建议 · 已勾选 {checked.length}</strong>
        <button disabled={busy} onClick={() => setChecked(preview.map(c => c.asset.id))}>勾选全部建议</button>
        <button disabled={busy} onClick={() => setChecked([])}>取消勾选</button>
        <button disabled={busy || !checked.length || dirty} onClick={() => void run(async () => {
          const count = await applyClassificationRule(draft, checked); setPreview(null); setChecked([]); await load(); await onChanged(); setMessage(`已新增 ${count} 个分类关联，可在批次记录中撤销`);
        })}>应用勾选建议</button>
      </div>
      {!preview.length && <p>没有新建议：资源不匹配、已具有标签、缺失，或曾被人工移除。</p>}
      <div className="rule-preview-list">{preview.slice(page * 100, (page + 1) * 100).map(candidate => <label key={candidate.asset.id}>
        <input type="checkbox" disabled={busy} checked={checked.includes(candidate.asset.id)} onChange={e => setChecked(prev => e.target.checked ? [...prev, candidate.asset.id] : prev.filter(id => id !== candidate.asset.id))} />
        <span><strong>{candidate.asset.file_name} → {tags.find(t => t.id === draft.tag_id)?.name}</strong><small>{candidate.reason}</small><small>{candidate.asset.absolute_path}</small></span>
      </label>)}</div>
      {preview.length > 100 && <div><button disabled={page === 0 || busy} onClick={() => setPage(p => p - 1)}>上一页建议</button><span>{page + 1} / {Math.ceil(preview.length / 100)}</span><button disabled={(page + 1) * 100 >= preview.length || busy} onClick={() => setPage(p => p + 1)}>下一页建议</button></div>}
    </section>}
    <details><summary>最近 30 批应用记录与撤销</summary><p>撤销只移除本批引入且未被人工重新添加或改名接管的关联。</p>
      {batches.map(batch => <div className="rule-actions" key={batch.id}><span>{batch.rule_name} · {new Date(batch.created_at).toLocaleString()} · 新增 {batch.added_count} 项</span><button disabled={busy || batch.undone} onClick={() => void run(async () => { const n = await undoClassificationBatch(batch.id); setPreview(null); setChecked([]); await load(); await onChanged(); setMessage(`已撤销 ${n} 个仍由本批管理的关联`); })}>{batch.undone ? "已撤销" : "撤销本批"}</button></div>)}
    </details>
  </section></div>;
}
