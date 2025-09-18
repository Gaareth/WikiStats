import { z } from "zod";

const statsSchema = z.object({
    bfs_sample_stats: z.union([z.record(z.any()), z.null()]).optional(),
    bi_bfs_sample_stats: z.union([z.record(z.any()), z.null()]).optional(),
    created_at: z.number().int().describe("utc timestamp"),
    dump_date: z.string(),
        longest_name: z.record(z.any()),
    longest_name_no_redirect: z.record(
        z.object({
            page_id: z.number().int().gte(0),
            page_title: z.string(),
            wiki_name: z.string(),
        }),
    ),
    max_num_links: z.tuple([z.string(), z.number().int().gte(0)]),
    max_num_pages: z.tuple([z.string(), z.number().int().gte(0)]),
    min_num_links: z.tuple([z.string(), z.number().int().gte(0)]),
    min_num_pages: z.tuple([z.string(), z.number().int().gte(0)]),
    most_linked: z.record(
        z.array(
            z.object({
                count: z.number().int().gte(0),
                page_id: z.number().int().gte(0),
                page_title: z.string(),
                wiki_name: z.string(),
            }),
        ),
    ),
    most_links: z.record(
        z.array(
            z.object({
                count: z.number().int().gte(0),
                page_id: z.number().int().gte(0),
                page_title: z.string(),
                wiki_name: z.string(),
            }),
        ),
    ),
    num_dead_orphan_pages: z.record(z.number().int().gte(0)),
    num_dead_pages: z.record(z.number().int().gte(0)),
    num_linked_redirects: z.record(z.number().int().gte(0)),
    num_links: z.record(z.number().int().gte(0)),
    num_orphan_pages: z.record(z.number().int().gte(0)),
    num_pages: z.record(z.number().int().gte(0)),
    num_redirects: z.record(z.number().int().gte(0)),
    seconds_taken: z.number().int().gte(0),
    wiki_sizes: z.object({
        sizes: z.array(
            z.object({
                compressed_selected_tables_size: z.number().int().gte(0),
                compressed_total_size: z.number().int().gte(0),
                decompressed_size: z
                    .union([z.number().int().gte(0), z.null()])
                    .optional(),
                name: z.string(),
                processed_size: z
                    .union([z.number().int().gte(0), z.null()])
                    .optional(),
            }),
        ),
        tables: z.array(z.string()),
    }),
    wikis: z.array(z.string()),
});

export default statsSchema;

// export type Stats = z.infer<typeof statsSchema>;


