import { index, integer, sqliteTable, text } from "drizzle-orm/sqlite-core";

export const wiki = sqliteTable("Wiki", {
    name: text("name").primaryKey(),
});

export const wikiPage = sqliteTable(
    "WikiPage",
    {
        pageId: integer("page_id").notNull(),
        pageTitle: text("page_title").notNull(),
        isRedirect: integer("is_redirect").notNull(),
    },
    (table) => {
        return {
            idxPageId: index("idx_page_id").on(table.pageId),
            idxTitleId: index("idx_title_id").on(table.pageTitle),
        };
    },
);

export const wikiLink = sqliteTable(
    "WikiLink",
    {
        pageId: integer("page_id"),
        pageLink: integer("page_link"),
    },
    (table) => {
        return {
            idxLinkPage: index("idx_link_page").on(table.pageLink),
            idxLinkId: index("idx_link_id").on(table.pageId),
        };
    },
);

export const spLink = sqliteTable("SP_Link", {
    sourceId: integer("source_id").notNull(),
    previousId: integer("previous_id").notNull(),
    pageId: integer("page_id").notNull(),
});

export type WikiLink = typeof wikiLink.$inferSelect; // return type when queried
export type WikiPage = typeof wikiPage.$inferSelect; // insert type
export type spLink = typeof spLink.$inferSelect; // return type when queried
