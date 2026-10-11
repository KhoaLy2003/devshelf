import { createHashRouter } from "react-router";
import { routes } from "./routes";

export { routes };
export const router = createHashRouter(routes);
