import { Link } from "react-router";

export function NotFoundScreen() {
  return (
    <div>
      <h1 className="m-0 mb-2 text-3xl font-semibold">Page not found</h1>
      <p className="m-0 mb-4 text-ink-muted">That page does not exist in DevShelf.</p>
      <Link to="/home" className="text-accent underline">
        Go to Home
      </Link>
    </div>
  );
}
