import { useEffect, useRef, useState, type RefObject } from "react";
import type { Asset } from "../types/asset";

export function mediaPreviewUrl(assetId: number): string {
  if (!Number.isSafeInteger(assetId) || assetId <= 0) {
    throw new Error("asset id must be a positive integer");
  }
  return `http://asset-media.localhost/${assetId}`;
}

type Props = {
  asset: Asset;
  onPlaybackStarted: (asset: Asset) => void;
};

const SUPPORTED_AUDIO = new Set(["mp3", "wav", "ogg"]);
const SUPPORTED_VIDEO = new Set(["mp4", "webm"]);

function isSupportedMedia(asset: Asset): boolean {
  const ext = asset.extension.toLowerCase();
  if (asset.asset_type === "audio") return SUPPORTED_AUDIO.has(ext);
  if (asset.asset_type === "video") return SUPPORTED_VIDEO.has(ext);
  return false;
}

export function MediaPreview({ asset, onPlaybackStarted }: Props) {
  const mediaRef = useRef<HTMLMediaElement>(null);
  const sessionRecordedRef = useRef(false);
  const assetRef = useRef(asset);
  const callbackRef = useRef(onPlaybackStarted);
  const [errorAssetId, setErrorAssetId] = useState<number | null>(null);

  assetRef.current = asset;
  callbackRef.current = onPlaybackStarted;

  useEffect(() => {
    const element = mediaRef.current;
    if (!element) return undefined;

    setErrorAssetId(null);
    sessionRecordedRef.current = false;

    const currentAssetId = asset.id;

    const handlePlay = () => {
      const currentAsset = assetRef.current;
      if (currentAsset.id !== currentAssetId) return;
      if (sessionRecordedRef.current) return;
      sessionRecordedRef.current = true;
      callbackRef.current(currentAsset);
    };

    const handlePause = () => {
      sessionRecordedRef.current = false;
    };

    const handleEnded = () => {
      sessionRecordedRef.current = false;
    };

    const handleError = () => {
      if (assetRef.current.id !== currentAssetId) return;
      setErrorAssetId(currentAssetId);
    };

    element.addEventListener("play", handlePlay);
    element.addEventListener("pause", handlePause);
    element.addEventListener("ended", handleEnded);
    element.addEventListener("error", handleError);

    return () => {
      element.pause();
      element.removeAttribute("src");
      element.load();
      element.removeEventListener("play", handlePlay);
      element.removeEventListener("pause", handlePause);
      element.removeEventListener("ended", handleEnded);
      element.removeEventListener("error", handleError);
    };
  }, [asset.id, asset.asset_type, asset.extension]);

  if (errorAssetId === asset.id) {
    return (
      <div className="media-preview-fallback">
        当前文件或编码无法在应用内预览。
      </div>
    );
  }

  if (!isSupportedMedia(asset)) {
    return (
      <div className="media-preview-fallback">
        此格式暂不支持应用内预览
      </div>
    );
  }

  const src = mediaPreviewUrl(asset.id);

  if (asset.asset_type === "video") {
    return (
      <video
        ref={mediaRef as RefObject<HTMLVideoElement>}
        src={src}
        controls
        preload="metadata"
        data-testid="media-preview"
        className="media-preview-video"
      />
    );
  }

  return (
    <audio
      ref={mediaRef as RefObject<HTMLAudioElement>}
      src={src}
      controls
      preload="metadata"
      data-testid="media-preview"
      className="media-preview-audio"
    />
  );
}
