import { Search } from "lucide-react";
import type { SearchScope } from "../types/asset";

type Props = {
  query: string;
  scope: SearchScope;
  onQueryChange: (query: string) => void;
  onScopeChange: (scope: SearchScope) => void;
};

export function SearchToolbar({ query, scope, onQueryChange, onScopeChange }: Props) {
  function toggle(key: keyof SearchScope) {
    onScopeChange({ ...scope, [key]: !scope[key] });
  }

  const scopeItems: { key: keyof SearchScope; label: string }[] = [
    { key: "fileName", label: "文件名" },
    { key: "tag", label: "标签" },
    { key: "note", label: "备注" },
    { key: "path", label: "路径" },
  ];

  return (
    <header className="search-toolbar">
      <div className="search-input-wrap">
        <Search className="search-input-icon" size={16} aria-hidden="true" />
        <input
          value={query}
          onChange={(event) => onQueryChange(event.target.value)}
          placeholder="搜索资源..."
          aria-label="搜索资源"
        />
      </div>
      <div className="scope-segments" aria-label="搜索范围">
        {scopeItems.map(({ key, label }) => (
          <button
            key={key}
            type="button"
            className={scope[key] ? "scope-segment active" : "scope-segment"}
            aria-pressed={scope[key]}
            onClick={() => toggle(key)}
          >
            {label}
          </button>
        ))}
      </div>
    </header>
  );
}
