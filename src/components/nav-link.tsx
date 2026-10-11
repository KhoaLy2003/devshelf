import type { ReactNode } from "react";
import { Link, useLocation } from "react-router";
import { isNavActive } from "../lib/nav";

interface NavLinkProps {
  path: string;
  children: ReactNode;
}

export function NavLink({ path, children }: NavLinkProps) {
  const { pathname } = useLocation();
  const active = isNavActive(pathname, path);

  return (
    <Link
      to={path}
      replace={active}
      aria-current={active ? "page" : undefined}
      className={`block rounded-lg px-3 py-2 text-sm font-medium no-underline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent ${
        active ? "bg-surface-raised text-ink" : "text-ink-muted hover:bg-surface hover:text-ink"
      }`}
    >
      {children}
    </Link>
  );
}
