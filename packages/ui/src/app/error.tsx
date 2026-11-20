"use client";

import { useEffect } from "react";

export default function Error({ error, reset }: { error: Error & { digest?: string }; reset: () => void }) {
  useEffect(() => {
    console.error(error);
  }, [error]);

  return (
    <main className="flex min-h-screen flex-col items-center justify-center gap-4 p-6 text-center">
      <h1 className="text-2xl font-semibold">Something went wrong</h1>
      <p className="max-w-md text-sm opacity-70">The page failed to render. You can try again.</p>
      <button type="button" onClick={reset} className="rounded border px-4 py-2 text-sm">
        Try again
      </button>
    </main>
  );
}
