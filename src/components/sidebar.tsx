import { NAV_ITEMS } from "../lib/nav";
import { NavLink } from "./nav-link";

export function Sidebar() {
  return (
    <aside className="flex h-full flex-col gap-6 border-r border-hairline bg-canvas px-4 py-6">
      <div className="px-3 text-lg font-semibold text-ink">DevShelf</div>
      <nav aria-label="Main">
        <ul className="m-0 flex list-none flex-col gap-1 p-0">
          {NAV_ITEMS.map((item) => (
            <li key={item.path}>
              <NavLink path={item.path}>{item.label}</NavLink>
            </li>
          ))}
        </ul>
      </nav>
    </aside>
  );
}
