import { z } from "zod";

export default z.object({
    bfs_sample_stats: z
        .union([
            z.record(
                z.object({
                    avg_depth_histogram: z
                        .record(
                            z.union([
                                z.object({
                                    avg_occurences: z.number(),
                                    std_dev: z.number(),
                                }),
                                z.never(),
                            ]),
                        )
                        .superRefine((value, ctx) => {
                            for (const key in value) {
                                let evaluated = false;
                                if (key.match(new RegExp("^\\d+$"))) {
                                    evaluated = true;
                                    const result = z
                                        .object({
                                            avg_occurences: z.number(),
                                            std_dev: z.number(),
                                        })
                                        .safeParse(value[key]);
                                    if (!result.success) {
                                        ctx.addIssue({
                                            path: [...ctx.path, key],
                                            code: "custom",
                                            message: `Invalid input: Key matching regex /${key}/ must match schema`,
                                            params: {
                                                issues: result.error.issues,
                                            },
                                        });
                                    }
                                }
                                if (!evaluated) {
                                    const result = z
                                        .never()
                                        .safeParse(value[key]);
                                    if (!result.success) {
                                        ctx.addIssue({
                                            path: [...ctx.path, key],
                                            code: "custom",
                                            message: `Invalid input: must match catchall schema`,
                                            params: {
                                                issues: result.error.issues,
                                            },
                                        });
                                    }
                                }
                            }
                        }),
                    deep_stat: z.object({
                        avg: z.number(),
                        max: z.array(z.any()).min(2).max(2),
                        min: z.array(z.any()).min(2).max(2),
                    }),
                    num_visited_map: z
                        .record(z.union([z.string(), z.never()]))
                        .superRefine((value, ctx) => {
                            for (const key in value) {
                                let evaluated = false;
                                if (key.match(new RegExp("^\\d+$"))) {
                                    evaluated = true;
                                    const result = z
                                        .string()
                                        .safeParse(value[key]);
                                    if (!result.success) {
                                        ctx.addIssue({
                                            path: [...ctx.path, key],
                                            code: "custom",
                                            message: `Invalid input: Key matching regex /${key}/ must match schema`,
                                            params: {
                                                issues: result.error.issues,
                                            },
                                        });
                                    }
                                }
                                if (!evaluated) {
                                    const result = z
                                        .never()
                                        .safeParse(value[key]);
                                    if (!result.success) {
                                        ctx.addIssue({
                                            path: [...ctx.path, key],
                                            code: "custom",
                                            message: `Invalid input: must match catchall schema`,
                                            params: {
                                                issues: result.error.issues,
                                            },
                                        });
                                    }
                                }
                            }
                        }),
                    path_depth_map: z
                        .record(z.union([z.array(z.string()), z.never()]))
                        .superRefine((value, ctx) => {
                            for (const key in value) {
                                let evaluated = false;
                                if (key.match(new RegExp("^\\d+$"))) {
                                    evaluated = true;
                                    const result = z
                                        .array(z.string())
                                        .safeParse(value[key]);
                                    if (!result.success) {
                                        ctx.addIssue({
                                            path: [...ctx.path, key],
                                            code: "custom",
                                            message: `Invalid input: Key matching regex /${key}/ must match schema`,
                                            params: {
                                                issues: result.error.issues,
                                            },
                                        });
                                    }
                                }
                                if (!evaluated) {
                                    const result = z
                                        .never()
                                        .safeParse(value[key]);
                                    if (!result.success) {
                                        ctx.addIssue({
                                            path: [...ctx.path, key],
                                            code: "custom",
                                            message: `Invalid input: must match catchall schema`,
                                            params: {
                                                issues: result.error.issues,
                                            },
                                        });
                                    }
                                }
                            }
                        }),
                    sample_size: z.number().int().gte(0),
                    seconds_taken: z.number().int().gte(0),
                    visit_stat: z.object({
                        avg: z.number(),
                        max: z.array(z.any()).min(2).max(2),
                        min: z.array(z.any()).min(2).max(2),
                    }),
                }),
            ),
            z.null(),
        ])
        .optional(),
    bi_bfs_sample_stats: z
        .union([
            z.record(
                z.object({
                    longest_path_stat: z.object({
                        avg: z.number(),
                        max: z.array(z.any()).min(2).max(2),
                        min: z.array(z.any()).min(2).max(2),
                    }),
                    path_length_histogram: z
                        .record(z.union([z.number().int().gte(0), z.never()]))
                        .superRefine((value, ctx) => {
                            for (const key in value) {
                                let evaluated = false;
                                if (key.match(new RegExp("^\\d+$"))) {
                                    evaluated = true;
                                    const result = z
                                        .number()
                                        .int()
                                        .gte(0)
                                        .safeParse(value[key]);
                                    if (!result.success) {
                                        ctx.addIssue({
                                            path: [...ctx.path, key],
                                            code: "custom",
                                            message: `Invalid input: Key matching regex /${key}/ must match schema`,
                                            params: {
                                                issues: result.error.issues,
                                            },
                                        });
                                    }
                                }
                                if (!evaluated) {
                                    const result = z
                                        .never()
                                        .safeParse(value[key]);
                                    if (!result.success) {
                                        ctx.addIssue({
                                            path: [...ctx.path, key],
                                            code: "custom",
                                            message: `Invalid input: must match catchall schema`,
                                            params: {
                                                issues: result.error.issues,
                                            },
                                        });
                                    }
                                }
                            }
                        }),
                    sample_size: z.number().int().gte(0),
                    seconds_taken: z.number().int().gte(0),
                }),
            ),
            z.null(),
        ])
        .optional(),
    created_at: z.number().int().describe("utc timestamp"),
    dump_date: z.string(),
    longest_name: z.record(
        z.object({
            page_id: z.number().int().gte(0),
            page_title: z.string(),
            wiki_name: z.string(),
        }),
    ),
    longest_name_no_redirect: z.record(
        z.object({
            page_id: z.number().int().gte(0),
            page_title: z.string(),
            wiki_name: z.string(),
        }),
    ),
    max_num_links: z.array(z.any()).min(2).max(2),
    max_num_pages: z.array(z.any()).min(2).max(2),
    min_num_links: z.array(z.any()).min(2).max(2),
    min_num_pages: z.array(z.any()).min(2).max(2),
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
                compressed_selected_tables_size: z
                    .union([z.number().int().gte(0), z.null()])
                    .optional(),
                compressed_total_size: z
                    .union([z.number().int().gte(0), z.null()])
                    .optional(),
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
