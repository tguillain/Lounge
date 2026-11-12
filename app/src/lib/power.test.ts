import { describe, expect, it } from "vitest";
import { POWER_CHOICES, confirmationOf } from "./power";

describe("power menu", () => {
  it("offers sleep first", () => {
    expect(POWER_CHOICES.map((choice) => choice.id)).toEqual(["sleep", "restart", "shut-down", "quit"]);
  });

  it("asks before restarting, shutting down or quitting, not before sleeping", () => {
    expect(confirmationOf("sleep")).toBeNull();
    expect(confirmationOf("restart")?.action).toBe("Restart");
    expect(confirmationOf("shut-down")?.title).toBe("Shut down the PC?");
    expect(confirmationOf("quit")?.action).toBe("Quit Lounge");
  });
});
