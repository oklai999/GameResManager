import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { LibrarySidebar } from "./LibrarySidebar";
import type { Collection } from "../types/asset";

function makeFolder(id: number, name: string) {
  return { id, name, path: `C:/${name.toLowerCase()}`, created_at: "", last_scanned_at: null, is_enabled: true };
}

function makeCollection(id: number, name: string): Collection {
  return { id, name, description: "" };
}

describe("LibrarySidebar", () => {
  it("calls onPickFolder when add folder button is clicked", async () => {
    const onPickFolder = vi.fn();
    render(
      <LibrarySidebar
        folders={[]}
        collections={[]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={vi.fn()}
        onSelectCollection={vi.fn()}
        onPickFolder={onPickFolder}
        onScanFolder={vi.fn()}
        onCancelScan={vi.fn()}
        onDeleteFolder={vi.fn()}
        onCreateCollection={vi.fn()}
        isScanning={false}
        latestJobs={{}}
        settingsPanel={null}
      />
    );

    await userEvent.click(screen.getByText("添加文件夹"));

    expect(onPickFolder).toHaveBeenCalled();
  });

  it("calls onScanFolder when scan button is clicked", async () => {
    const onScanFolder = vi.fn();
    render(
      <LibrarySidebar
        folders={[makeFolder(1, "Assets")]}
        collections={[]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={vi.fn()}
        onSelectCollection={vi.fn()}
        onPickFolder={vi.fn()}
        onScanFolder={onScanFolder}
        onCancelScan={vi.fn()}
        onDeleteFolder={vi.fn()}
        onCreateCollection={vi.fn()}
        isScanning={false}
        latestJobs={{}}
        settingsPanel={null}
      />
    );

    await userEvent.click(screen.getByText("扫描"));

    expect(onScanFolder).toHaveBeenCalledWith(1);
  });

  it("calls onSelectFolder when folder row is clicked", async () => {
    const onSelectFolder = vi.fn();
    render(
      <LibrarySidebar
        folders={[makeFolder(1, "Assets")]}
        collections={[]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={onSelectFolder}
        onSelectCollection={vi.fn()}
        onPickFolder={vi.fn()}
        onScanFolder={vi.fn()}
        onCancelScan={vi.fn()}
        onDeleteFolder={vi.fn()}
        onCreateCollection={vi.fn()}
        isScanning={false}
        latestJobs={{}}
        settingsPanel={null}
      />
    );

    await userEvent.click(screen.getByTitle("C:/assets"));

    expect(onSelectFolder).toHaveBeenCalledWith(1);
  });

  it("clicking scan button does not trigger row selection", async () => {
    const onSelectFolder = vi.fn();
    const onScanFolder = vi.fn();
    render(
      <LibrarySidebar
        folders={[makeFolder(1, "Assets")]}
        collections={[]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={onSelectFolder}
        onSelectCollection={vi.fn()}
        onPickFolder={vi.fn()}
        onScanFolder={onScanFolder}
        onCancelScan={vi.fn()}
        onDeleteFolder={vi.fn()}
        onCreateCollection={vi.fn()}
        isScanning={false}
        latestJobs={{}}
        settingsPanel={null}
      />
    );

    await userEvent.click(screen.getByText("扫描"));

    expect(onScanFolder).toHaveBeenCalledWith(1);
    expect(onSelectFolder).not.toHaveBeenCalled();
  });

  it("disables scan button when isScanning is true", () => {
    render(
      <LibrarySidebar
        folders={[makeFolder(1, "Assets")]}
        collections={[]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={vi.fn()}
        onSelectCollection={vi.fn()}
        onPickFolder={vi.fn()}
        onScanFolder={vi.fn()}
        onCancelScan={vi.fn()}
        onDeleteFolder={vi.fn()}
        onCreateCollection={vi.fn()}
        isScanning={true}
        latestJobs={{}}
        settingsPanel={null}
      />
    );

    const btn = screen.getByText("扫描") as HTMLButtonElement;
    expect(btn.disabled).toBe(true);
  });

  it("deselects folder on second click", async () => {
    const onSelectFolder = vi.fn();
    render(
      <LibrarySidebar
        folders={[makeFolder(1, "Assets")]}
        collections={[]}
        activeFilter="all"
        selectedFolderId={1}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={onSelectFolder}
        onSelectCollection={vi.fn()}
        onPickFolder={vi.fn()}
        onScanFolder={vi.fn()}
        onCancelScan={vi.fn()}
        onDeleteFolder={vi.fn()}
        onCreateCollection={vi.fn()}
        isScanning={false}
        latestJobs={{}}
        settingsPanel={null}
      />
    );

    await userEvent.click(screen.getByTitle("C:/assets"));

    expect(onSelectFolder).toHaveBeenCalledWith(null);
  });

  it("shows collection names and calls onSelectCollection on click", async () => {
    const onSelectCollection = vi.fn();
    render(
      <LibrarySidebar
        folders={[]}
        collections={[makeCollection(1, "Heroes"), makeCollection(2, "Icons")]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={vi.fn()}
        onSelectCollection={onSelectCollection}
        onPickFolder={vi.fn()}
        onScanFolder={vi.fn()}
        onCancelScan={vi.fn()}
        onDeleteFolder={vi.fn()}
        onCreateCollection={vi.fn()}
        isScanning={false}
        latestJobs={{}}
        settingsPanel={null}
      />
    );

    expect(screen.getByText("Heroes")).toBeInTheDocument();
    expect(screen.getByText("Icons")).toBeInTheDocument();

    await userEvent.click(screen.getByText("Heroes"));

    expect(onSelectCollection).toHaveBeenCalledWith(1);
  });

  it("calls onCreateCollection when form is submitted with non-empty name", async () => {
    const onCreateCollection = vi.fn();
    render(
      <LibrarySidebar
        folders={[]}
        collections={[]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={vi.fn()}
        onSelectCollection={vi.fn()}
        onPickFolder={vi.fn()}
        onScanFolder={vi.fn()}
        onCancelScan={vi.fn()}
        onDeleteFolder={vi.fn()}
        onCreateCollection={onCreateCollection}
        isScanning={false}
        latestJobs={{}}
        settingsPanel={null}
      />
    );

    const input = screen.getByPlaceholderText("新建集合...");
    await userEvent.type(input, "My Set");
    await userEvent.click(screen.getByText("创建"));

    expect(onCreateCollection).toHaveBeenCalledWith("My Set");
  });

  it("confirms folder index removal without implying disk file deletion", async () => {
    const confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(false);
    render(
      <LibrarySidebar
        folders={[makeFolder(1, "Assets")]}
        collections={[]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={vi.fn()}
        onSelectCollection={vi.fn()}
        onPickFolder={vi.fn()}
        onScanFolder={vi.fn()}
        onCancelScan={vi.fn()}
        onDeleteFolder={vi.fn()}
        onCreateCollection={vi.fn()}
        isScanning={false}
        latestJobs={{}}
        settingsPanel={null}
      />
    );

    await userEvent.click(screen.getByTitle("从资源库移除索引"));

    expect(confirmSpy).toHaveBeenCalledWith(
      "确定要从资源库移除「Assets」的索引吗？这只会删除应用内索引和关联整理数据，不会删除磁盘上的原始文件。"
    );
    expect(screen.queryByTitle("删除文件夹")).not.toBeInTheDocument();

    confirmSpy.mockRestore();
  });
});
