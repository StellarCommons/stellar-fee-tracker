import Link from "next/link";

export default function NotFound() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-center gap-4 p-6 text-center">
      <h1 className="text-2xl font-semibold">Page not found</h1>
      <p className="max-w-md text-sm opacity-70">That URL does not exist.</p>
      <Link href="/dashboard" className="rounded border px-4 py-2 text-sm">
        Go to the dashboard
      </Link>
    </main>
  );
}
