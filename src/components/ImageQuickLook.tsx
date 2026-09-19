import { useEffect, useRef, useState } from "react";
import type { Asset, LibraryFolder } from "../types/asset";

type Props = {
  assets: Asset[];
  initialId: number;
  folders: LibraryFolder[];
  onClose: () => void;
  onSelect: (id: number) => void;
  onViewed: (asset: Asset) => void;
};

export function ImageQuickLook({ assets, initialId, folders, onClose, onSelect, onViewed }: Props) {
  const [index, setIndex] = useState(Math.max(0, assets.findIndex(a => a.id === initialId)));
  const [zoom, setZoom] = useState<number | null>(null);
  const [loaded, setLoaded] = useState(false);
  const [failed, setFailed] = useState(false);
  const [size, setSize] = useState({ width: 0, height: 0 });
  const dialog = useRef<HTMLDivElement>(null);
  const viewport = useRef<HTMLDivElement>(null);
  const drag = useRef<{ x: number; y: number; left: number; top: number } | null>(null);
  const recorded = useRef(false);
  const asset = assets[index];
  const folder = folders.find(f => f.id === asset.library_folder_id);
  const normalized = asset.absolute_path.replace(/\\/g, "/");
  const root = folder?.path.replace(/\\/g, "/").replace(/\/$/, "");
  const relative = root && normalized.startsWith(root + "/") ? normalized.slice(root.length + 1) : normalized;

  useEffect(() => {
    const previous = document.activeElement as HTMLElement | null;
    dialog.current?.focus();
    return () => { if (previous?.isConnected) previous.focus({ preventScroll: true }); };
  }, []);

  function move(next: number) {
    if (next < 0 || next >= assets.length) return;
    setIndex(next); setZoom(null); setLoaded(false); setFailed(false); recorded.current = false;
    if (viewport.current) { viewport.current.scrollTop = 0; viewport.current.scrollLeft = 0; }
    onSelect(assets[next].id);
  }

  return <div className="quicklook-backdrop">
    <div className="quicklook-dialog" ref={dialog} role="dialog" aria-modal="true" aria-label="原图快看" tabIndex={-1}
      onKeyDown={e => {
        if (e.key === "Escape") { e.preventDefault(); onClose(); }
        if (e.key === "ArrowLeft") { e.preventDefault(); move(index - 1); }
        if (e.key === "ArrowRight") { e.preventDefault(); move(index + 1); }
        if (e.key === "Tab") {
          const buttons = Array.from(dialog.current?.querySelectorAll<HTMLButtonElement>("button:not(:disabled)") ?? []);
          const first = buttons[0], last = buttons[buttons.length - 1];
          if (e.shiftKey && (document.activeElement === first || document.activeElement === dialog.current)) { e.preventDefault(); last?.focus(); }
          else if (!e.shiftKey && (document.activeElement === last || document.activeElement === dialog.current)) { e.preventDefault(); first?.focus(); }
        }
      }}>
      <header><strong>{asset.file_name}</strong><span>{loaded ? `${size.width} × ${size.height}` : "原图"} · {index + 1} / {assets.length} 个已加载候选</span><button onClick={onClose}>关闭 (Esc)</button></header>
      <div className="quicklook-tools">
        <button disabled={index === 0} onClick={() => move(index - 1)}>上一张 ←</button>
        <button disabled={index === assets.length - 1} onClick={() => move(index + 1)}>下一张 →</button>
        <button onClick={() => setZoom(null)}>适应窗口</button>
        <button onClick={() => setZoom(1)}>100%</button>
        <button disabled={!loaded} onClick={() => setZoom(z => Math.max(0.1, (z ?? 1) / 1.5))}>缩小</button>
        <button disabled={!loaded} onClick={() => setZoom(z => Math.min(8, (z ?? 1) * 1.5))}>放大</button>
        <span>{zoom === null ? "适应窗口" : `${Math.round(zoom * 100)}%`} · 放大后拖动查看</span>
      </div>
      <div className="quicklook-viewport" ref={viewport}
        onPointerDown={e => { if (e.button !== 0 || !viewport.current) return; e.currentTarget.setPointerCapture(e.pointerId); drag.current = { x: e.clientX, y: e.clientY, left: viewport.current.scrollLeft, top: viewport.current.scrollTop }; }}
        onPointerMove={e => { if (!drag.current) return; e.currentTarget.scrollLeft = drag.current.left - e.clientX + drag.current.x; e.currentTarget.scrollTop = drag.current.top - e.clientY + drag.current.y; }}
        onPointerUp={() => { drag.current = null; }} onPointerCancel={() => { drag.current = null; }}>
        {failed ? <p role="alert">无法快看：文件缺失、格式不支持，或超过 32 MB / 1600 万像素。支持 PNG、JPEG、WebP、GIF（首帧）、BMP。</p> : <>
          {!loaded && <p role="status">正在读取原图…</p>}
          <img key={asset.id} src={`http://asset-image.localhost/${asset.id}`} alt={asset.file_name} draggable={false}
            className={zoom === null ? "quicklook-fit" : "quicklook-scaled"}
            style={zoom === null ? undefined : { width: size.width * zoom, height: size.height * zoom }}
            onError={() => setFailed(true)} onLoad={e => {
              setLoaded(true); setSize({ width: e.currentTarget.naturalWidth, height: e.currentTarget.naturalHeight });
              if (!recorded.current) { recorded.current = true; onViewed(asset); }
            }} />
        </>}
      </div>
      <footer title={asset.absolute_path}>{folder?.name} / {relative}</footer>
    </div>
  </div>;
}
