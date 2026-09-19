import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, expect, it, vi } from "vitest";
import { ClassificationRules } from "./ClassificationRules";
import * as api from "../api/discovery";
import type { AssetSearchRequest, Asset } from "../types/asset";
vi.mock("../api/discovery", () => ({
  listClassificationRules: vi.fn(), listClassificationBatches: vi.fn(), saveClassificationRule: vi.fn(),
  previewClassificationRule: vi.fn(), applyClassificationRule: vi.fn(), undoClassificationBatch: vi.fn(),
}));
const rule: api.ClassificationRule = { id: 1, name: "角色规则", field: "directory", pattern: "Archer", tag_id: 2 };
const candidates: api.RuleCandidate[] = [{ asset: { id: 10, file_name: "Idle.png", absolute_path: "C:/Archer/Idle.png" } as Asset, reason: "目录段完全匹配「Archer」" }];
const request = { query: "", limit: 200, offset: 0 } as AssetSearchRequest;
const tags: api.FacetTag[] = [{ id: 2, name: "角色", dimension: "usage", asset_count: 0, color: "#123456" }];
beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(api.listClassificationRules).mockResolvedValue([rule]);
  vi.mocked(api.listClassificationBatches).mockResolvedValue([{ id: 3, rule_name: "旧批次", created_at: "2026-09-19T00:00:00Z", added_count: 1, undone: false }]);
  vi.mocked(api.previewClassificationRule).mockResolvedValue(candidates);
  vi.mocked(api.applyClassificationRule).mockResolvedValue(1);
  vi.mocked(api.undoClassificationBatch).mockResolvedValue(1);
});
it("requires preview and explicit selection before applying, then allows batch undo", async () => {
  const changed = vi.fn(async () => {});
  render(<ClassificationRules tags={tags} request={request} onClose={vi.fn()} onChanged={changed} />);
  const preview = await screen.findByRole("button", { name: "预览当前查找范围" });
  await waitFor(() => expect(preview).toBeEnabled());
  expect(screen.queryByRole("button", { name: "应用勾选建议" })).not.toBeInTheDocument();
  await userEvent.click(preview);
  const apply = await screen.findByRole("button", { name: "应用勾选建议" });
  expect(apply).toBeDisabled();
  await userEvent.click(screen.getByRole("checkbox"));
  await userEvent.click(apply);
  await waitFor(() => expect(api.applyClassificationRule).toHaveBeenCalledWith(rule, [10]));
  await screen.findByText("已新增 1 个分类关联，可在批次记录中撤销");
  await userEvent.click(screen.getByText("最近 30 批应用记录与撤销"));
  await userEvent.click(screen.getByRole("button", { name: "撤销本批" }));
  await waitFor(() => expect(api.undoClassificationBatch).toHaveBeenCalledWith(3));
  expect(changed).toHaveBeenCalled();
});
it("discards a preview response when search scope changed while it was loading", async () => {
  let resolve!: (value: api.RuleCandidate[]) => void;
  vi.mocked(api.previewClassificationRule).mockReturnValue(new Promise(r => { resolve = r; }));
  const props = { tags, request, onClose: vi.fn(), onChanged: vi.fn(async () => {}) };
  const { rerender } = render(<ClassificationRules {...props} />);
  const preview = await screen.findByRole("button", { name: "预览当前查找范围" });
  await waitFor(() => expect(preview).toBeEnabled());
  await userEvent.click(preview);
  rerender(<ClassificationRules {...props} request={{ ...request, query: "new scope" }} />);
  resolve(candidates);
  await waitFor(() => expect(screen.queryByText("处理中…")).not.toBeInTheDocument());
  expect(screen.queryByRole("button", { name: "应用勾选建议" })).not.toBeInTheDocument();
});
it("invalidates reviewed candidates when editing a rule and reports preview errors", async () => {
  render(<ClassificationRules tags={tags} request={request} onClose={vi.fn()} onChanged={vi.fn(async () => {})} />);
  const preview = await screen.findByRole("button", { name: "预览当前查找范围" });
  await waitFor(() => expect(preview).toBeEnabled());
  vi.mocked(api.previewClassificationRule).mockRejectedValueOnce(new Error("请缩小范围"));
  await userEvent.click(preview); await screen.findByText("请缩小范围");
  await userEvent.click(preview); await screen.findByRole("button", { name: "应用勾选建议" });
  fireEvent.change(screen.getByLabelText("规则匹配词"), { target: { value: "Units" } });
  expect(preview).toBeDisabled();
  expect(screen.queryByRole("button", { name: "应用勾选建议" })).not.toBeInTheDocument();
  expect(api.applyClassificationRule).not.toHaveBeenCalled();
});
