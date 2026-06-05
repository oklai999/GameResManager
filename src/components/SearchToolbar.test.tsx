import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { SearchToolbar } from "./SearchToolbar";
import type { SearchScope } from "../types/asset";

const scope: SearchScope = {
  fileName: true,
  tag: true,
  note: true,
  path: false,
};

describe("SearchToolbar", () => {
  it("uses segmented scope buttons that toggle the search scope", async () => {
    const onScopeChange = vi.fn();
    render(
      <SearchToolbar
        query=""
        scope={scope}
        onQueryChange={vi.fn()}
        onScopeChange={onScopeChange}
      />
    );

    const pathButton = screen.getByRole("button", { name: "路径" });
    expect(pathButton).toHaveAttribute("aria-pressed", "false");

    await userEvent.click(pathButton);

    expect(onScopeChange).toHaveBeenCalledWith({ ...scope, path: true });
  });
});
