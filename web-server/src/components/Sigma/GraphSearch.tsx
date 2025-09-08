import clsx from "clsx";
import Fuse from "fuse.js";
import { For, createSignal } from "solid-js";
import type { Pages } from "../../pages/api/[wiki_name]/pages";
import { TooltipButton } from "../TooltipButton";
import GraphPanel from "./GraphPanel";
import { useSigma } from "./SigmaContext";

interface SearchProps {
    onChange: (node: string) => void;
}

const SearchIcon = () => (
    <svg
        xmlns="http://www.w3.org/2000/svg"
        width="24"
        height="24"
        viewBox="0 0 24 24"
    >
        <path
            fill="currentColor"
            d="m19.6 21l-6.3-6.3q-.75.6-1.725.95T9.5 16q-2.725 0-4.612-1.888T3 9.5t1.888-4.612T9.5 3t4.613 1.888T16 9.5q0 1.1-.35 2.075T14.7 13.3l6.3 6.3zM9.5 14q1.875 0 3.188-1.312T14 9.5t-1.312-3.187T9.5 5T6.313 6.313T5 9.5t1.313 3.188T9.5 14"
        />
    </svg>
);

export default function GraphSearch(props: SearchProps) {
    const [pages, setPages] = createSignal<string[]>([]);
    const [searchTerm, setSearchTerm] = createSignal<string>();
    const [notFound, setNotFound] = createSignal(false);

    const { getGraph, setGraph } = useSigma()!["graph"];

    const fetchPages = async (prefix: string) => {
        const resp = await fetch(`/api/dewiki/pages?prefix=${prefix}`);
        const json: Pages = await resp.json();
        return json;
    };

    const updateDatalist = async (prefix: string) => {
        setNotFound(false);
        const MAX_ELEMENTS = 20;
        // setPages([]);

        // const page_titles = (await fetchPages(prefix)).map((p) => p.pageTitle);
        // const page_titles = props.graphLabels();
        const page_titles = getGraph().mapNodes((n) =>
            getGraph().getNodeAttribute(n, "label"),
        );

        const fuse = new Fuse(page_titles);
        const result = fuse
            .search(prefix)
            .slice(0, MAX_ELEMENTS)
            .map(({ item }) => item);

        setPages(result);
        setSearchTerm(prefix);
    };

    const searchNode = (e: Event) => {
        e.preventDefault();
        if (searchTerm() === undefined) {
            return;
        }

        if (!pages().includes(searchTerm()!)) {
            setNotFound(true);
        } else {
            setNotFound(false);
        }

        props.onChange(searchTerm()!);
    };

    return (
        <GraphPanel title="Search" class="z-10">
            <form class="flex gap-1 justify-between" onSubmit={searchNode}>
                <input
                    type="text"
                    placeholder="Search for a node"
                    class={clsx(
                        "dark-layer-2 dark:text-slate-50 w-full input-default dark-layer-2",
                        notFound() &&
                            "!border-amber-600 !focus:ring !ring-amber-600 !outline-none",
                    )}
                    list="datalist"
                    onInput={(e) => updateDatalist(e.target.value)}
                />
                <TooltipButton
                    class="button dark-layer-2 rounded-none px-2 py-1 group relative"
                    // disabled={!searchTerm()}
                    aria-labelledby="search-tip"
                    tooltip="Search"
                >
                    <SearchIcon />
                </TooltipButton>
            </form>
            <datalist id="datalist">
                <For each={pages()}>
                    {(page) => <option value={page}></option>}
                </For>
            </datalist>
        </GraphPanel>
    );
}
