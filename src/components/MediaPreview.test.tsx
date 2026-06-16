import { render, screen, fireEvent } from "@testing-library/react";
import { afterAll, beforeAll, describe, expect, it, vi } from "vitest";
import type { Asset } from "../types/asset";
import { MediaPreview, mediaPreviewUrl } from "./MediaPreview";

function makeAsset(overrides: Partial<Asset> = {}): Asset {
  return {
    id: 1,
    library_folder_id: 1,
    absolute_path: "C:\\assets\\sample.bin",
    file_name: "sample.bin",
    extension: "bin",
    asset_type: "other",
    file_size: 1024,
    modified_at: "2026-06-16T00:00:00Z",
    width: null,
    height: null,
    thumbnail_path: null,
    thumbnail_status: "none",
    thumbnail_error: null,
    note: "",
    is_favorite: false,
    is_missing: false,
    created_at: "2026-06-16T00:00:00Z",
    updated_at: "2026-06-16T00:00:00Z",
    ...overrides,
  };
}

const audioAsset = makeAsset({
  id: 42,
  asset_type: "audio",
  extension: "mp3",
  file_name: "sample.mp3",
});

const videoAsset = makeAsset({
  id: 43,
  asset_type: "video",
  extension: "mp4",
  file_name: "sample.mp4",
});

beforeAll(() => {
  vi.spyOn(HTMLMediaElement.prototype, "pause").mockImplementation(() => {});
  vi.spyOn(HTMLMediaElement.prototype, "load").mockImplementation(() => {});
});

afterAll(() => {
  vi.restoreAllMocks();
});

describe("mediaPreviewUrl", () => {
  it("returns the Windows custom protocol origin URL", () => {
    expect(mediaPreviewUrl(266197)).toBe("http://asset-media.localhost/266197");
  });

  it.each([0, -1, 1.5, Number.NaN, Number.POSITIVE_INFINITY])(
    "rejects non-positive-integer id %s",
    (assetId) => {
      expect(() => mediaPreviewUrl(assetId as number)).toThrow(
        "asset id must be a positive integer"
      );
    }
  );
});

describe("MediaPreview formats", () => {
  it.each([
    ["audio", "mp3", "audio"],
    ["audio", "wav", "audio"],
    ["audio", "ogg", "audio"],
    ["video", "mp4", "video"],
    ["video", "webm", "video"],
  ] as const)(
    "renders %s %s with native controls",
    (assetType, extension, role) => {
      render(
        <MediaPreview
          asset={makeAsset({ asset_type: assetType, extension })}
          onPlaybackStarted={vi.fn()}
        />
      );
      const media = screen.getByTestId("media-preview");
      expect(media.tagName.toLowerCase()).toBe(role);
      expect(media).toHaveAttribute("controls");
      expect(media).toHaveAttribute("preload", "metadata");
    }
  );

  it("shows placeholder for unsupported formats", () => {
    render(
      <MediaPreview
        asset={makeAsset({ asset_type: "audio", extension: "flac" })}
        onPlaybackStarted={vi.fn()}
      />
    );
    expect(
      screen.getByText("此格式暂不支持应用内预览")
    ).toBeInTheDocument();
  });
});

describe("MediaPreview lifecycle", () => {
  it("records once per continuous play session", () => {
    const onStarted = vi.fn();
    render(<MediaPreview asset={audioAsset} onPlaybackStarted={onStarted} />);
    const media = screen.getByTestId("media-preview");
    fireEvent.play(media);
    fireEvent.play(media);
    expect(onStarted).toHaveBeenCalledTimes(1);
  });

  it("records again after pause then play", () => {
    const onStarted = vi.fn();
    render(<MediaPreview asset={audioAsset} onPlaybackStarted={onStarted} />);
    const media = screen.getByTestId("media-preview");
    fireEvent.play(media);
    fireEvent.pause(media);
    fireEvent.play(media);
    expect(onStarted).toHaveBeenCalledTimes(2);
  });

  it("records again after ended then play", () => {
    const onStarted = vi.fn();
    render(<MediaPreview asset={audioAsset} onPlaybackStarted={onStarted} />);
    const media = screen.getByTestId("media-preview");
    fireEvent.play(media);
    fireEvent.ended(media);
    fireEvent.play(media);
    expect(onStarted).toHaveBeenCalledTimes(2);
  });

  it("stops and resets the old media when the asset changes", () => {
    const pause = vi.spyOn(HTMLMediaElement.prototype, "pause");
    const load = vi.spyOn(HTMLMediaElement.prototype, "load");
    const removeAttribute = vi.spyOn(Element.prototype, "removeAttribute");
    const { rerender } = render(
      <MediaPreview asset={audioAsset} onPlaybackStarted={vi.fn()} />
    );
    rerender(<MediaPreview asset={videoAsset} onPlaybackStarted={vi.fn()} />);
    expect(pause).toHaveBeenCalled();
    expect(load).toHaveBeenCalled();
    expect(removeAttribute).toHaveBeenCalledWith("src");
    pause.mockRestore();
    load.mockRestore();
    removeAttribute.mockRestore();
  });

  it("shows a decoding fallback after media error", () => {
    render(<MediaPreview asset={videoAsset} onPlaybackStarted={vi.fn()} />);
    const media = screen.getByTestId("media-preview");
    fireEvent.error(media);
    expect(
      screen.getByText("当前文件或编码无法在应用内预览。")
    ).toBeInTheDocument();
  });

  it("recovers the media element when switching away from an errored asset", () => {
    const { rerender } = render(
      <MediaPreview asset={videoAsset} onPlaybackStarted={vi.fn()} />
    );
    fireEvent.error(screen.getByTestId("media-preview"));
    expect(
      screen.getByText("当前文件或编码无法在应用内预览。")
    ).toBeInTheDocument();

    rerender(<MediaPreview asset={audioAsset} onPlaybackStarted={vi.fn()} />);
    const media = screen.getByTestId("media-preview");
    expect(media.tagName.toLowerCase()).toBe("audio");
    expect(
      screen.queryByText("当前文件或编码无法在应用内预览。")
    ).not.toBeInTheDocument();
  });

  it("does not record delayed events from the previous asset as the new asset", () => {
    const onStarted = vi.fn();
    const addEventListener = vi.spyOn(
      HTMLMediaElement.prototype,
      "addEventListener"
    );
    const { rerender } = render(
      <MediaPreview asset={audioAsset} onPlaybackStarted={onStarted} />
    );
    const playHandler = addEventListener.mock.calls.find(
      ([type]) => type === "play"
    )?.[1] as EventListener;
    addEventListener.mockClear();

    rerender(<MediaPreview asset={videoAsset} onPlaybackStarted={onStarted} />);

    // Simulate a stale play event dispatching through the old handler.
    playHandler?.(new Event("play"));
    expect(onStarted).not.toHaveBeenCalled();
    addEventListener.mockRestore();
  });
});
