// Streamed while the dashboard's server component waits on the backend API.
export default function Loading() {
  return (
    <main className="flex min-h-screen items-center justify-center p-6" aria-busy="true">
      <p role="status" className="animate-pulse text-sm opacity-70">
        Loading fee data…
      </p>
    </main>
  );
}
