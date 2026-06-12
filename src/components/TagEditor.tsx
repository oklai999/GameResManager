import { useEffect, useState } from "react";
import { listTags } from "../api/tauri";

type Props = {
  existingTags: string[];
  recentTags?: string[];
  onApply: (tagName: string) => void;
  onRemove: (tagName: string) => void;
};

export function TagEditor({ existingTags, recentTags = [], onApply, onRemove }: Props) {
  const [input, setInput] = useState("");
  const [allTags, setAllTags] = useState<string[]>([]);

  useEffect(() => {
    listTags()
      .then((tags) => setAllTags(tags.map((t) => t.name)))
      .catch(() => setAllTags([]));
  }, []);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const name = input.trim();
    if (name) {
      onApply(name);
      setInput("");
    }
  };

  const normalizedInput = input.trim().toLowerCase();
  const existingSet = new Set(existingTags.map((tag) => tag.toLowerCase()));
  const suggestions = [...new Set([...recentTags, ...allTags])]
    .filter((tag) => !existingSet.has(tag.toLowerCase()))
    .filter((tag) => normalizedInput.length === 0 || tag.toLowerCase().includes(normalizedInput))
    .slice(0, 8);

  function applySuggestion(tag: string) {
    onApply(tag);
    setInput("");
  }

  return (
    <div className="tag-editor">
      <div className="tag-chips">
        {existingTags.map((tag) => (
          <span key={tag} className="tag-chip">
            <span>{tag}</span>
            <button
              type="button"
              className="tag-chip-remove"
              aria-label={`移除标签 ${tag}`}
              onClick={() => onRemove(tag)}
            >
              ×
            </button>
          </span>
        ))}
      </div>
      <form onSubmit={handleSubmit} className="tag-form">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="输入标签..."
          className="tag-input"
        />
        <button type="submit" className="tag-apply-btn">添加</button>
      </form>
      {suggestions.length > 0 && (
        <div className="tag-suggestions">
          {suggestions.map((tag) => (
            <button
              key={tag}
              type="button"
              className="tag-suggestion"
              onClick={() => applySuggestion(tag)}
            >
              {tag}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
