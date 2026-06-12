import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { NavigationRail } from "./NavigationRail";

describe("NavigationRail", () => {
  it("marks the active workspace section", () => {
    render(
      <NavigationRail
        activeSection="library"
        sidebarOpen
        onSelect={vi.fn()}
      />
    );
    expect(screen.getByRole("button", { name: "资源库" }))
      .toHaveAttribute("aria-pressed", "true");
  });

  it("selects a different workspace section", async () => {
    const onSelect = vi.fn();
    render(
      <NavigationRail
        activeSection="library"
        sidebarOpen
        onSelect={onSelect}
      />
    );
    await userEvent.click(screen.getByRole("button", { name: "标签" }));
    expect(onSelect).toHaveBeenCalledWith("tags");
  });

  it("allows the current section to collapse the contextual sidebar", async () => {
    const onSelect = vi.fn();
    render(
      <NavigationRail
        activeSection="library"
        sidebarOpen
        onSelect={onSelect}
      />
    );
    await userEvent.click(screen.getByRole("button", { name: "资源库" }));
    expect(onSelect).toHaveBeenCalledWith("library");
  });
});
