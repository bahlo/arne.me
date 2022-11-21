import Link from "next/link";

const navigationRoutes = [
  { path: "/", name: "Home" },
  { path: "/work", name: "Work" },
  { path: "/blog", name: "Blog" },
  { path: "/weekly", name: "Weekly" },
];

export default function Header() {
  return (
    <header>
      <nav>
        <ul>
          {navigationRoutes.map((route) => {
            const isActive = false; // TODO: Figure out how to determine if the route is active
            return (
              <li key={route.path}>
                <Link href={route.path} className={isActive ? "active" : ""}>
                  {route.name}
                </Link>
              </li>
            );
          })}
        </ul>
      </nav>
    </header>
  );
}
