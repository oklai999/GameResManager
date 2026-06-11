import { useEffect, useState } from "react";
import { Trash2, X } from "lucide-react";
import type { Collection } from "../types/asset";

type Props = {
  collections: Collection[];
  onClose: () => void;
  onUpdate: (collectionId: number, name: string, description: string) => void;
  onDelete: (collectionId: number) => void;
};

export function CollectionManager({
  collections,
  onClose,
  onUpdate,
  onDelete,
}: Props) {
  const [selectedId, setSelectedId] = useState<number | null>(
    collections[0]?.id ?? null
  );
  const selected =
    collections.find((collection) => collection.id === selectedId) ?? null;
  const [name, setName] = useState(selected?.name ?? "");
  const [description, setDescription] = useState(selected?.description ?? "");
  const [confirmingDelete, setConfirmingDelete] = useState(false);

  useEffect(() => {
    setSelectedId((prev) => {
      const exists = collections.some((c) => c.id === prev);
      if (exists) return prev;
      return collections[0]?.id ?? null;
    });
  }, [collections]);

  useEffect(() => {
    setName(selected?.name ?? "");
    setDescription(selected?.description ?? "");
    setConfirmingDelete(false);
  }, [selected?.id, selected?.name, selected?.description]);

  return (
    <div className="collection-manager-backdrop" role="presentation">
      <section
        className="collection-manager-panel"
        role="dialog"
        aria-modal="true"
        aria-label="集合管理"
      >
        <header className="collection-manager-header">
          <div>
            <h2>集合管理</h2>
            <p>只管理应用内整理数据，不会改动磁盘上的原始素材。</p>
          </div>
          <button type="button" onClick={onClose} aria-label="关闭集合管理">
            <X size={16} aria-hidden="true" />
          </button>
        </header>

        {collections.length === 0 ? (
          <div className="collection-manager-empty">还没有创建集合。</div>
        ) : (
          <div className="collection-manager-body">
            <nav className="collection-manager-list" aria-label="集合列表">
              {collections.map((collection) => (
                <button
                  type="button"
                  key={collection.id}
                  className={collection.id === selectedId ? "active" : ""}
                  onClick={() => setSelectedId(collection.id)}
                >
                  <span>{collection.name}</span>
                  <small>{collection.asset_count}</small>
                </button>
              ))}
            </nav>

            {selected && (
              <form
                className="collection-manager-form"
                onSubmit={(event) => {
                  event.preventDefault();
                  const nextName = name.trim();
                  if (nextName) {
                    onUpdate(selected.id, nextName, description.trim());
                  }
                }}
              >
                <label>
                  集合名称
                  <input
                    aria-label="集合名称"
                    value={name}
                    onChange={(event) => setName(event.target.value)}
                  />
                </label>
                <label>
                  集合描述
                  <textarea
                    aria-label="集合描述"
                    rows={4}
                    value={description}
                    onChange={(event) => setDescription(event.target.value)}
                  />
                </label>
                <div className="collection-manager-actions">
                  <button type="submit" disabled={!name.trim()}>
                    保存集合
                  </button>
                  <button
                    type="button"
                    className="danger"
                    onClick={() => setConfirmingDelete(true)}
                  >
                    <Trash2 size={14} aria-hidden="true" />
                    删除集合
                  </button>
                </div>
                {confirmingDelete && (
                  <div className="collection-delete-confirm">
                    <p>
                      只会删除集合和成员关系，不会删除任何原始素材文件。
                    </p>
                    <button
                      type="button"
                      className="danger"
                      onClick={() => onDelete(selected.id)}
                    >
                      确认删除集合
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
