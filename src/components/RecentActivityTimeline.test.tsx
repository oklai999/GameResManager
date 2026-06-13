import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { RecentActivityTimeline } from "./RecentActivityTimeline";
import type { Asset, RecentActivityItem } from "../types/asset";

const asset: Asset = {
  id: 1,
  library_folder_id: 1,
  absolute_path: "C:/assets/sound.wav",
  file_name: "sound.wav",
  extension: "wav",
  asset_type: "audio",
  file_size: 100,
  modified_at: "2026-05-27T00:00:00Z",
  width: null,
  height: null,
  thumbnail_path: null,
  thumbnail_status: "none" as const,
  thumbnail_error: null,
  note: "",
  is_favorite: false,
  is_missing: false,
  created_at: "2026-05-27T00:00:00Z",
  updated_at: "2026-05-27T00:00:00Z",
};

const item: RecentActivityItem = {
  asset,
  latest_action_type: "open_file",
  latest_action_at: "2026-06-13T10:00:00Z",
  action_count: 2,
  open_file_count: 1,
  reveal_folder_count: 0,
  copy_path_count: 1,
  preview_media_count: 0,
  actions: [
    { id: 1, asset_id: 1, action_type: "copy_path", created_at: "2026-06-13T09:00:00Z" },
    { id: 2, asset_id: 1, action_type: "open_file", created_at: "2026-06-13T10:00:00Z" },
  ],
};

const missingItem: RecentActivityItem = {
  ...item,
  asset: { ...asset, id: 2, file_name: "missing.wav", absolute_path: "C:/assets/missing.wav", is_missing: true },
};

const defaultProps = {
  items: [item],
  selectedIds: [] as number[],
  totalCount: 1,
  loading: false,
  loadingMore: false,
  canLoadMore: false,
  onSelectAsset: vi.fn(),
  onOpenAsset: vi.fn(),
  onLoadMore: vi.fn(),
};

describe("RecentActivityTimeline", () => {
  it("selects a resource without expanding details", async () => {
    const onSelect = vi.fn();
    render(<RecentActivityTimeline {...defaultProps} onSelectAsset={onSelect} />);

    await userEvent.click(screen.getByRole("button", { name: "选择 sound.wav" }));

    expect(onSelect).toHaveBeenCalledWith(item.asset.id);
    expect(screen.queryByText("复制路径")).not.toBeInTheDocument();
  });

  it("expands activity details independently", async () => {
    const { container } = render(<RecentActivityTimeline {...defaultProps} />);

    await userEvent.click(screen.getByRole("button", { name: "展开 sound.wav 的活动明细" }));

    const details = container.querySelector(".recent-activity-details");
    expect(details).toBeInTheDocument();
    expect(details?.textContent).toContain("复制路径");
    expect(details?.textContent).toContain("打开文件");
  });

  it("collapses expanded activity details", async () => {
    render(<RecentActivityTimeline {...defaultProps} />);

    await userEvent.click(screen.getByRole("button", { name: "展开 sound.wav 的活动明细" }));
    expect(screen.getByText("复制路径")).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "收起 sound.wav 的活动明细" }));
    expect(screen.queryByText("复制路径")).not.toBeInTheDocument();
  });

  it("opens available assets on double click and blocks missing assets", async () => {
    const onOpen = vi.fn();
    const { rerender } = render(<RecentActivityTimeline {...defaultProps} onOpenAsset={onOpen} />);

    fireEvent.doubleClick(screen.getByRole("button", { name: "选择 sound.wav" }));
    expect(onOpen).toHaveBeenCalledWith(item.asset);

    rerender(<RecentActivityTimeline {...defaultProps} items={[missingItem]} onOpenAsset={onOpen} />);

    fireEvent.doubleClick(screen.getByRole("button", { name: "选择 missing.wav" }));
    expect(onOpen).toHaveBeenCalledTimes(1);
  });

  it("loads another page", async () => {
    const onLoadMore = vi.fn();
    render(<RecentActivityTimeline {...defaultProps} canLoadMore onLoadMore={onLoadMore} />);

    await userEvent.click(screen.getByRole("button", { name: "加载更多活动" }));

    expect(onLoadMore).toHaveBeenCalled();
  });

  it("shows relative time and action summary", () => {
    const { container } = render(<RecentActivityTimeline {...defaultProps} />);

    expect(screen.getByText("sound.wav")).toBeInTheDocument();
    expect(container.querySelector(".recent-activity-summary")?.textContent).toMatch(/打开文件/);
    expect(container.querySelector(".recent-activity-summary")?.textContent).toMatch(/2 次/);
  });

  it("marks selected card", () => {
    const { container } = render(<RecentActivityTimeline {...defaultProps} selectedIds={[1]} />);

    const card = container.querySelector(".recent-activity-card.selected");
    expect(card).toBeInTheDocument();
  });

  it("does not show load more when canLoadMore is false", () => {
    render(<RecentActivityTimeline {...defaultProps} canLoadMore={false} />);

    expect(screen.queryByRole("button", { name: "加载更多活动" })).not.toBeInTheDocument();
  });

  it("disables load more while loadingMore", () => {
    render(<RecentActivityTimeline {...defaultProps} canLoadMore loadingMore />);

    expect(screen.getByRole("button", { name: "加载更多活动" })).toBeDisabled();
  });
});
