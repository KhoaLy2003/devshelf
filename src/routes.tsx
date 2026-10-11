import { Navigate, type RouteObject } from "react-router";
import { AppShell } from "./components/app-shell";
import { ConfigScreen } from "./screens/config-screen";
import { HomeScreen } from "./screens/home-screen";
import { NotesScreen } from "./screens/notes-screen";
import { NotFoundScreen } from "./screens/not-found-screen";
import { ProjectDetailScreen } from "./screens/project-detail-screen";
import { ProjectsScreen } from "./screens/projects-screen";
import { SettingsScreen } from "./screens/settings-screen";
import { VaultScreen } from "./screens/vault-screen";
import { WorkspacesScreen } from "./screens/workspaces-screen";

export const routes: RouteObject[] = [
  {
    id: "shell",
    path: "/",
    element: <AppShell />,
    children: [
      { id: "root", index: true, element: <Navigate to="/home" replace /> },
      { id: "home", path: "home", element: <HomeScreen /> },
      { id: "projects", path: "projects", element: <ProjectsScreen /> },
      { id: "project-detail", path: "projects/:id", element: <ProjectDetailScreen /> },
      { id: "workspaces", path: "workspaces", element: <WorkspacesScreen /> },
      { id: "config", path: "config", element: <ConfigScreen /> },
      { id: "vault", path: "vault", element: <VaultScreen /> },
      { id: "notes", path: "notes", element: <NotesScreen /> },
      { id: "settings", path: "settings", element: <SettingsScreen /> },
      { id: "not-found", path: "*", element: <NotFoundScreen /> },
    ],
  },
];
