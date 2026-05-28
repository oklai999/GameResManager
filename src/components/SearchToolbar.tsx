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

  return (
    <header className="search-toolbar">
      <input
        value={query}
        onChange={(event) => onQueryChange(event.target.value)}
        placeholder="搜索文件名、标签、备注或路径"
      />
      <label><input type="checkbox" checked={scope.fileName} onChange={() => toggle("fileName")} /> 文件名</label>
      <label><input type="checkbox" checked={scope.tag} onChange={() => toggle("tag")} /> 标签</label>
      <label><input type="checkbox" checked={scope.note} onChange={() => toggle("note")} /> 备注</label>
      <label><input type="checkbox" checked={scope.path} onChange={() => toggle("path")} /> 路径</label>
    </header>
  );
}
