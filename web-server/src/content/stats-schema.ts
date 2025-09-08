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
    wikis: z.array(z.string()),
});

export default statsSchema;

// export type Stats = z.infer<typeof statsSchema>;

export type Stats = {
    bfs_sample_stats: BfsSample | null;
    bi_bfs_sample_stats: BiBfsSample | null;
    created_at: number; // utc timestamp (int64)
    dump_date: string;
    longest_name: Record<string, Page>;
    longest_name_no_redirect: Record<string, Page>;
    max_num_links: [string, number]; // [string, uint64]
    max_num_pages: [string, number]; // [string, uint64]
    min_num_links: [string, number]; // [string, uint64]
    min_num_pages: [string, number]; // [string, uint64]
    most_linked: Record<string, LinkCount[]>;
    most_links: Record<string, LinkCount[]>;
    num_dead_orphan_pages: Record<string, number>; // uint64
    num_dead_pages: Record<string, number>; // uint64
    num_linked_redirects: Record<string, number>; // uint64
    num_links: Record<string, number>; // uint64
    num_orphan_pages: Record<string, number>; // uint64
    num_pages: Record<string, number>; // uint64
    num_redirects: Record<string, number>; // uint64
    wikis: string[];
};

type BfsSample = {
    avg_depth_histogram: Record<string, number>;
    deep_stat: MaxMinAvgForStringAndUint32;
    sample_size: number; // uint32
    visit_stat: MaxMinAvgForStringAndUint32;
};

type BiBfsSample = {
    longest_path_stat: MaxMinAvgForTupleOfStringAndStringAndUint32;
    path_length_histogram: Record<string, number>; // uint64
    sample_size: number; // uint32
};

type LinkCount = {
    count: number; // uint64
    page_id: number; // uint64
    page_title: string;
    wiki_name: string;
};

type MaxMinAvgForStringAndUint32 = {
    avg: number;
    max: [string, number]; // [string, uint32]
    min: [string, number]; // [string, uint32]
};

type MaxMinAvgForTupleOfStringAndStringAndUint32 = {
    avg: number;
    max: [[string, string], number]; // [[string, string], uint32]
    min: [[string, string], number]; // [[string, string], uint32]
};

type Page = {
    page_id: number; // uint64
    page_title: string;
    wiki_name: string;
};
