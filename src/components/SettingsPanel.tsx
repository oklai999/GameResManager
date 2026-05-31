import { useCallback, useState } from "react";
import type { ScanSettings } from "../types/asset";

type Props = {
  settings: ScanSettings;
  onChange: (settings: ScanSettings) => void;
};

export function SettingsPanel({ settings, onChange }: Props) {
  const [localDirs, setLocalDirs] = useState(settings.ignored_directory_names);
  const [localExts, setLocalExts] = useState(settings.ignored_extensions);

  const commitDirs = useCallback(() => {
    if (localDirs !== settings.ignored_directory_names) {
      onChange({ ...settings, ignored_directory_names: localDirs });
    }
  }, [localDirs, settings, onChange]);

  const commitExts = useCallback(() => {
    if (localExts !== settings.ignored_extensions) {
      onChange({ ...settings, ignored_extensions: localExts });
    }
  }, [localExts, settings, onChange]);

  const toggle = (key: keyof ScanSettings) => {
    onChange({ ...settings, [key]: !settings[key] });
  };

  const checkboxes: { key: keyof ScanSettings; label: string }[] = [
    { key: "include_images", label: "图片" },
    { key: "include_audio", label: "音频" },
    { key: "include_video", label: "视频" },
    { key: "include_fonts", label: "字体" },
    { key: "include_models", label: "3D 模型" },
    { key: "include_spine", label: "Spine" },
    { key: "include_psd", label: "PSD 索引" },
    { key: "generate_psd_thumbnails", label: "PSD 缩略图" },
  ];

  return (
    <div className="scan-settings-panel">
      <div className="panel-heading secondary">设置</div>

      <div className="scan-settings-grid">
        {checkboxes.map(({ key, label }) => (
          <label key={key} className="scan-setting-row">
            <input
              type="checkbox"
              checked={!!settings[key]}
              onChange={() => toggle(key)}
            />
            <span>{label}</span>
          </label>
        ))}
      </div>

      <div className="scan-setting-readonly">
        <div className="muted">忽略目录（逗号分隔）</div>
        <textarea
          className="note-textarea"
          rows={3}
          value={localDirs}
          onChange={(e) => setLocalDirs(e.target.value)}
          onBlur={commitDirs}
          style={{ marginTop: 6, fontSize: 12 }}
        />
      </div>

      <div className="scan-setting-readonly">
        <div className="muted">忽略扩展名（逗号、空格或换行分隔，自动去点前缀）</div>
        <textarea
          className="note-textarea"
          rows={2}
          value={localExts}
          onChange={(e) => setLocalExts(e.target.value)}
          onBlur={commitExts}
          style={{ marginTop: 6, fontSize: 12 }}
        />
      </div>

      <div className="scan-setting-readonly">
        <div className="muted">数据库路径（当前版本仅展示，不支持在界面迁移）</div>
        <div className="scan-ignored-dirs" style={{ marginTop: 6 }}>
          {settings.database_path ?? "默认"}
        </div>
      </div>

      <div className="scan-setting-readonly">
        <div className="muted">缩略图缓存路径（当前版本仅展示，不支持在界面迁移）</div>
        <div className="scan-ignored-dirs" style={{ marginTop: 6 }}>
          {settings.thumbnail_cache_dir ?? "默认"}
        </div>
      </div>
    </div>
  );
}
