import { useEffect, useState } from "react";
import { Tag } from "lucide-react";
import { Trash2, X } from "lucide-react";
import type { Tag as TagType } from "../types/asset";

type Props = {
  tags: TagType[];
  onClose: () => void;
  onUpdate: (tagId: number, name: string, color: string) => void;
  onDelete: (tagId: number) => void;
};

export function TagManager({ tags, onClose, onUpdate, onDelete }: Props) {
  const [selectedId, setSelectedId] = useState<number | null>(tags[0]?.id ?? null);
  const selected = tags.find((tag) => tag.id === selectedId) ?? null;
  const [name, setName] = useState(selected?.name ?? "");
  const [color, setColor] = useState(selected?.color ?? "#5B8DEF");
  const [confirmingDelete, setConfirmingDelete] = useState(false);

  useEffect(() => {
    setSelectedId((previous) =>
      tags.some((tag) => tag.id === previous) ? previous : tags[0]?.id ?? null
    );
  }, [tags]);

  useEffect(() => {
    setName(selected?.name ?? "");
    setColor(selected?.color ?? "#5B8DEF");
    setConfirmingDelete(false);
  }, [selected?.id, selected?.name, selected?.color]);

  const handleSubmit = (event: React.FormEvent) => {
    event.preventDefault();
    const nextName = name.trim();
    const nextColor = color.trim();
    if (selected && nextName && /^#[0-9a-fA-F]{6}$/.test(nextColor)) {
      onUpdate(selected.id, nextName, nextColor);
    }
  };

  return (
    <div className="collection-manager-backdrop" role="presentation">
      <section
        className="collection-manager-panel"
        role="dialog"
        aria-modal="true"
        aria-label="标签管理"
      >
        <header className="collection-manager-header">
          <div>
            <h2>标签管理</h2>
            <p>只管理应用内标签，不会改动磁盘上的原始素材。</p>
          </div>
          <button type="button" onClick={onClose} aria-label="关闭标签管理">
            <X size={16} aria-hidden="true" />
          </button>
        </header>

        {tags.length === 0 ? (
          <div className="collection-manager-empty">还没有标签。</div>
        ) : (
          <div className="collection-manager-body">
            <nav className="collection-manager-list" aria-label="标签列表">
              {tags.map((tag) => (
                <button
                  type="button"
                  key={tag.id}
                  className={tag.id === selectedId ? "active" : ""}
                  onClick={() => setSelectedId(tag.id)}
                >
                  <span className="tag-manager-list-label">
                    <span
                      className="tag-manager-color-dot"
                      style={{ backgroundColor: tag.color }}
                      aria-hidden="true"
                    />
                    <span>{tag.name}</span>
                  </span>
                  <small>{tag.asset_count}</small>
                </button>
              ))}
            </nav>

            {selected && (
              <form
                className="collection-manager-form"
                onSubmit={handleSubmit}
              >
                <label>
                  标签名称
                  <input
                    aria-label="标签名称"
                    value={name}
                    onChange={(event) => setName(event.target.value)}
                  />
                </label>
                <label>
                  标签颜色
                  <div className="tag-color-row">
                    <input
                      aria-label="标签颜色"
                      value={color}
                      onChange={(event) => setColor(event.target.value)}
                      maxLength={7}
                    />
                    <span
                      className="tag-manager-color-dot"
                      style={{ backgroundColor: color, width: 32, height: 32 }}
                      aria-hidden="true"
                    />
                  </div>
                </label>
                <div className="collection-manager-actions">
                  <button
                    type="submit"
                    disabled={!name.trim() || !/^#[0-9a-fA-F]{6}$/.test(color.trim())}
                  >
                    保存标签
                  </button>
                  <button
                    type="button"
                    className="danger"
                    onClick={() => setConfirmingDelete(true)}
                  >
                    <Trash2 size={14} aria-hidden="true" />
                    删除标签
                  </button>
                </div>
                {confirmingDelete && (
                  <div className="collection-delete-confirm">
                    <p>
                      只会删除应用内标签及资源关联，不会删除或修改原始素材文件。
                    </p>
                    <button
                      type="button"
                      className="danger"
                      onClick={() => onDelete(selected.id)}
                    >
                      确认删除标签
                    </button>
                    <button
                      type="button"
                      onClick={() => setConfirmingDelete(false)}
                    >
                      取消
                    </button>
                  </div>
                )}
              </form>
            )}
          </div>
        )}
      </section>
    </div>
  );
}
