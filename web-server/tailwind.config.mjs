/** @type {import('tailwindcss').Config} */
export default {
    content: ["./src/**/*.{astro,html,js,jsx,md,mdx,svelte,ts,tsx,vue}"],
    theme: {
        extend: {
            colors: {
                dark_00: "#121212",
                dark_01: "#1e1e1e",
                dark_02: "#242424",
                dark_03: "#2c2c2c",
                dark_04: "#333333",
                dark_05: "#383838",

                gradient_start: "rgb(79, 128, 227)",
            },
        },
    },
    plugins: [require("tailwind-hamburgers")],
    darkMode: "selector",
};
