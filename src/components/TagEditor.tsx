import { useEffect, useState } from "react";
import { listTags } from "../api/tauri";

type Props = {
  existingTags: string[];
  onApply: (tagName: string) => void;
};

export function TagEditor({ existingTags, onApply }: Props) {
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

  const suggestions = allTags.filter(
    (t) =>
      t.toLowerCase().includes(input.trim().toLowerCase()) &&
      !existingTags.includes(t)
  );

  return (
    <div className="tag-editor">
      <div className="tag-chips">
        {existingTags.map((tag) => (
          <span key={tag} className="tag-chip">
            {tag}
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
          {suggestions.slice(0, 8).map((tag) => (
            <button
              key={tag}
              type="button"
              className="tag-suggestion"
              onClick={() => onApply(tag)}
            >
              {tag}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
