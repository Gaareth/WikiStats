import { createRoot, createSignal } from "solid-js";

type Theme = "dark" | "light";

function createTheme() {
    const [theme, setTheme] = createSignal<Theme>("light");
    return { theme, setTheme };
}

export default createRoot(createTheme);
