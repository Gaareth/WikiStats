# WikiStats Webserver

> built using AstroJS

Live: [wiki-stats.gaareth.com](wiki-stats.gaareth.com)

## Setup

-   Generate databases and stats json files using the rust executables
-   Create `.env` according to `example.env`
-   `npm install`
-   `npm run dev`

## TODO

-   proper tooltip ui
-   num links histogram
-   search for links on mirror endpoint
-   PageLinks pagination, table, sorting, searching
-   dual slider visuals on ios safari
-   graphs add show link on mirror endpoint for edges
-   graph should indicate if its loading (see neighbor graph)
-   404 page for invalid wikis

## Notes

-   Prefer Client components over Astro components as they can be used anywhere. Astro components can only be used in `.astro` files.
-   Astro components are server rendered, so no client side interactivity unless you add `client:load` or similar directive. This is important for tooltip and their uuid generation.
