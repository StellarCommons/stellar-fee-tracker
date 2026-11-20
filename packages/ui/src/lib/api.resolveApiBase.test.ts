import { describe, expect, it } from "vitest";
import { resolveApiBase } from "@/lib/api";

describe("resolveApiBase", () => {
  it("uses NEXT_PUBLIC_API_URL in the browser", () => {
    expect(resolveApiBase(false, { NEXT_PUBLIC_API_URL: "/api" })).toBe("/api");
  });

  it("falls back to localhost in the browser", () => {
    expect(resolveApiBase(false, {})).toBe("http://localhost:8080");
  });

  it("prefers BACKEND_HOST on the server and adds https", () => {
    expect(resolveApiBase(true, { BACKEND_HOST: "api.example.com", BACKEND_URL: "http://x" })).toBe("https://api.example.com");
  });

  it("uses BACKEND_URL on the server when no host is set", () => {
    expect(resolveApiBase(true, { BACKEND_URL: "http://backend:8080" })).toBe("http://backend:8080");
  });

  it("accepts an absolute NEXT_PUBLIC_API_URL on the server", () => {
    expect(resolveApiBase(true, { NEXT_PUBLIC_API_URL: "https://api.example.com" })).toBe("https://api.example.com");
  });

  it("ignores a relative NEXT_PUBLIC_API_URL on the server", () => {
    expect(resolveApiBase(true, { NEXT_PUBLIC_API_URL: "/api" })).toBe("http://localhost:8080");
  });
});
