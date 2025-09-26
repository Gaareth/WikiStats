import type { z } from "astro/zod";
import { createEffect, createSignal, For } from "solid-js";
import type statsSchema from "../content/stats-schema";
import { cn, DBNAME_TO_SITEINFO, formatBytesIntl, WIKI_TYPES } from "../utils";
import Pill from "./Pill";
import { Table } from "./Table";

export function WikiSizesTable(props: {
    wikis_columns: NonNullable<
        z.infer<typeof statsSchema>["wiki_sizes"]
    >["sizes"];
    supported_wikis: string[];
    dump_date?: string;
}) {
    const [selection, setSelection] = createSignal(["wiki"]);
    const [data, setData] = createSignal(props.wikis_columns);

    const supported = () =>
        data().filter((entry) => props.supported_wikis.includes(entry.name));

    // filter data based on selection
    createEffect(() => {
        setData(
            props.wikis_columns.filter((entry) =>
                selection().some((type) => entry.name.endsWith(type)),
            ),
        );
    });

    const renderWikiLink = (item: string) => {
        const url = `https://dumps.wikimedia.org/${item}/${props.dump_date != null ? props.dump_date + "/" : ""}`;
        const siteinfo = DBNAME_TO_SITEINFO.get(item);
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
                <p>Supported Wikis ({supported().length})</p>
                <Table
                    data={supported()}
                    columns={[
                        {
                            key: "name",
                            label: "Wiki",
                            isSortable: true,
                            render: renderWikiLink,
                        },
                        {
                            key: "compressed_total_size",
                            label: "Total",
                            render: formatBytesIntl,
                            isSortable: true,
                        },
                        {
                            key: "compressed_selected_tables_size",
                            label: "Relevant",
                            render: formatBytesIntl,
                            isSortable: true,
                        },
                        {
                            key: "decompressed_size",
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
                    sortConfig={{ key: "processed_size", direction: "desc" }}
                />
            </div>

            <div>
                <p>
                    All
                    <span class="text-secondary">
                        {selection() != WIKI_TYPES ? "*" : ""}
                    </span>{" "}
                    Wikis ({data().length})
                </p>
                <Table
                    data={data()}
                    columns={[
                        {
                            key: "name",
                            label: "Wiki",
                            isSortable: true,
                            render: renderWikiLink,
                        },
                        {
                            key: "compressed_total_size",
                            label: "Total",
                            render: formatBytesIntl,
                            isSortable: true,
                        },
                        {
                            key: "compressed_selected_tables_size",
                            label: "Relevant",
                            render: formatBytesIntl,
                            isSortable: true,
                        },
                    ]}
                    sortConfig={{
                        key: "compressed_selected_tables_size",
                        direction: "desc",
                    }}
                />
            </div>
        </div>
    );
}
