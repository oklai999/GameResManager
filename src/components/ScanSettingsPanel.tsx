import type { ScanSettings } from "../types/asset";

type Props = {
  settings: ScanSettings;
  onChange: (settings: ScanSettings) => void;
};

export function ScanSettingsPanel({ settings, onChange }: Props) {
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
      <div className="panel-heading secondary">扫描规则</div>
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
        <div className="muted">忽略目录</div>
        <div className="scan-ignored-dirs">{settings.ignored_directory_names}</div>
      </div>
    </div>
  );
}
