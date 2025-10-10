import type { z } from "astro/zod";
import { createEffect, createSignal, For, Show } from "solid-js";
import { WIKI_TYPES } from "../constants";
import type statsSchema from "../schemas/stats-schema";
import { cn, formatBytesIntl } from "../utils";
import type { SiteInfo } from "../wiki-api";
import Pill from "./Pill";
import { Table } from "./Table";

export function WikiSizesTable(props: {
    web_wikis_columns: NonNullable<
        z.infer<typeof statsSchema>["web_wiki_sizes"]
    >["sizes"];
    local_wikis_columns?: NonNullable<
        z.infer<typeof statsSchema>["local_wiki_sizes"]
    >["sizes"];
    supported_wikis: string[];
    dump_date?: string;
    siteinfo: Map<string, SiteInfo>;
}) {
    const [selection, setSelection] = createSignal(["wiki"]);
    const [webSizes, setWebSizes] = createSignal(props.web_wikis_columns);
    const [localSizes, setLocalSizes] = createSignal(
        props.local_wikis_columns?.map((entry) => {
            const webSize = props.web_wikis_columns.find(
                (e) => e.name === entry.name,
            );
            return {
                ...entry,
                total_size: webSize?.total_size,
                selected_tables_size: webSize?.selected_tables_size,
            };
        }),
    );

    // filter data based on selection
    createEffect(() => {
        setWebSizes(
            props.web_wikis_columns.filter((entry) =>
                selection().some((type) => entry.name.endsWith(type)),
            ),
        );
    });

    const renderWikiLink = (item: string) => {
        const url = `https://dumps.wikimedia.org/${item}/${props.dump_date != null ? props.dump_date + "/" : ""}`;
        const siteinfo = props.siteinfo.get(item);
        return (
            <div class="flex flex-wrap items-center gap-1 sm:gap-2">
                {siteinfo != null && (
                    <div class="flex flex-wrap gap-1">
                        <a
                            href={siteinfo.url}
                            target="_blank"
                            rel="noopener noreferrer">
                            <Pill>{siteinfo?.name ?? siteinfo?.sitename}</Pill>
                        </a>
                        {siteinfo.localname != null &&
                            siteinfo.localname != siteinfo.name && (
                                <Pill>{siteinfo?.localname}</Pill>
                            )}
                    </div>
                )}
                <a href={url} target="_blank" rel="noopener noreferrer">
                    {item}
                </a>
            </div>
        );
    };

    return (
        <div class="flex flex-col gap-8">
            <div class="flex flex-col gap-2">
                <div class="flex flex-col sm:flex-row sm:items-end gap-1">
                    <p>Filter by Wiki type</p>
                    <div class="sm:ml-auto flex gap-2">
                        <button
                            type="button"
                            class="button dark-layer-1"
                            onClick={() => setSelection(WIKI_TYPES)}>
                            Select all
                        </button>
                        <button
                            type="button"
                            class="button dark-layer-1"
                            onClick={() => setSelection([])}>
                            Select none
                        </button>
                    </div>
                </div>

                <div
                    class="grid gap-1"
                    style={{
                        "grid-template-columns": `repeat(auto-fit, minmax(125px, 1fr))`,
                    }}>
                    <For each={WIKI_TYPES}>
                        {(type) => (
                            <button
                                role="radio"
                                onClick={() => {
                                    let newSelection = selection().slice();
                                    if (selection().includes(type)) {
                                        newSelection.splice(
                                            newSelection.indexOf(type),
                                            1,
                                        );
                                    } else {
                                        newSelection.push(type);
                                    }
                                    setSelection(newSelection);
                                }}
                                class={cn(
                                    "button-select",
                                    selection().includes(type) &&
                                        "button-select-selected",
                                )}>
                                {type}
                            </button>
                        )}
                    </For>
                </div>
            </div>

            <div>
                <Show
                    when={localSizes()}
                    fallback={<p>No data about downloaded wikis</p>}>
                    {(sizes) => (
                        <>
                            <p>Downloaded Wikis ({sizes().length})</p>
                            <Table
                                data={sizes()}
                                columns={[
                                    {
                                        key: "name",
                                        label: "Wiki",
                                        isSortable: true,
                                        render: renderWikiLink,
                                    },
                                    {
                                        key: "total_size",
                                        label: "Total",
                                        render: formatBytesIntl,
                                        isSortable: true,
                                    },
                                    {
                                        key: "selected_tables_size",
                                        label: "Relevant",
                                        render: formatBytesIntl,
                                        isSortable: true,
                                    },
                                    {
                                        key: "download_size",
                                        label: "Decompressed",
                                        render: formatBytesIntl,
                                        isSortable: true,
                                    },
                                    {
                                        key: "processed_size",
                                        label: "Processed",
                                        render: formatBytesIntl,
                                        isSortable: true,
                                    },
                                ]}
                                sortConfig={{
                                    key: "processed_size",
                                    direction: "desc",
                                }}
                            />
                        </>
                    )}
                </Show>
            </div>

            <div>
                <p>
                    All
                    <span class="text-secondary">
                        {selection() != WIKI_TYPES ? "*" : ""}
                    </span>{" "}
                    Wikis ({webSizes().length})
                </p>
                <Table
                    data={webSizes()}
                    columns={[
                        {
                            key: "name",
                            label: "Wiki",
                            isSortable: true,
                            render: renderWikiLink,
                        },
                        {
                            key: "total_size",
                            label: "Total",
                            render: formatBytesIntl,
                            isSortable: true,
                        },
                        {
                            key: "selected_tables_size",
                            label: "Relevant",
                            render: formatBytesIntl,
                            isSortable: true,
                        },
                    ]}
                    sortConfig={{
                        key: "selected_tables_size",
                        direction: "desc",
                    }}
                />
            </div>
        </div>
    );
}
