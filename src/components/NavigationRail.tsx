import {
  Clock3,
  FolderKanban,
  Layers3,
  Settings,
  Shapes,
  Tags,
} from "lucide-react";

export type WorkbenchSection =
  | "library"
  | "types"
  | "tags"
  | "collections"
  | "recent"
  | "settings";

type Props = {
  activeSection: WorkbenchSection;
  sidebarOpen: boolean;
  onSelect: (section: WorkbenchSection) => void;
};

const items = [
  { id: "library", label: "资源库", icon: FolderKanban },
  { id: "types", label: "类型", icon: Shapes },
  { id: "tags", label: "标签", icon: Tags },
  { id: "collections", label: "集合", icon: Layers3 },
  { id: "recent", label: "最近", icon: Clock3 },
  { id: "settings", label: "设置", icon: Settings },
] satisfies Array<{
  id: WorkbenchSection;
  label: string;
  icon: typeof FolderKanban;
}>;

export function NavigationRail({ activeSection, sidebarOpen, onSelect }: Props) {
  return (
    <nav className="navigation-rail" aria-label="工作区导航">
      <div className="navigation-rail-brand" aria-label="游戏资源管理器">
        GR
      </div>
      <div className="navigation-rail-items">
        {items.map(({ id, label, icon: Icon }) => {
          const active = activeSection === id && sidebarOpen;
          return (
            <button
              key={id}
              type="button"
              className={active ? "rail-button active" : "rail-button"}
              aria-label={label}
              aria-pressed={active}
              title={label}
              onClick={() => onSelect(id)}
            >
              <Icon size={18} aria-hidden="true" />
            </button>
          );
        })}
      </div>
    </nav>
  );
}
