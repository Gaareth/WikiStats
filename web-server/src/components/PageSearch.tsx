import clsx from "clsx";
import {
    createResource,
    createSignal,
    createUniqueId,
    ErrorBoundary,
    For,
    onMount,
    Show,
    Suspense,
} from "solid-js";
import { get_wiki_prefix, reinitializeFlowBiteTooltips } from "../utils";
import {
    fetchPagesWikipediaAPI,
    fetchRandomPage,
    type Page,
} from "../wiki-api";
import { LoadingSpinner, Refresh } from "./ClientIcons/Icons";
import FloatingLabelInput from "./FloatingLabelInput";
import PageLinkPills from "./PageLinkPills";
import { TooltipButton } from "./TooltipButton";

interface Props {
    wiki_name?: string;
    default_value?: string | null;
    name: string;
    placeholder: string;
    classNameWrapper?: string;
}

export function PageSearch(props: Props) {
    const [value, setValue] = createSignal(props.default_value);
    const [show, setShow] = createSignal(false);
    const [loading, setLoading] = createSignal(false);
    const [pages, setPages] = createSignal<Page[]>([]);
    const [error, setError] = createSignal<string>();
    const componentID = createUniqueId();
    let inputRef: HTMLInputElement | undefined;

    // const [pages] = createResource(value, async (prefix) =>
    //   fetchPages(props.wiki_name, prefix)
    // );

    const updateSuggestions = async (value: string) => {
        if (
            props.wiki_name === undefined ||
            value == null ||
            value.length == 0
        ) {
            setPages([]);
            return;
        }

        setLoading(true);
        try {
            const fetched_pages = await fetchPagesWikipediaAPI(
                props.wiki_name,
                value,
            );
            setError(undefined);

            // const fetched_pages = await fetchPages(props.wiki_name, e.target.value);
            setPages(fetched_pages);
        } catch (e) {
            setError("Failed fetching suggestions: " + (e as Error));
        } finally {
            setLoading(false);
        }
    };

    onMount(async () => {
        const v = value();
        if (v != null && v.length > 0) {
            updateSuggestions(v);
        }
    });

    function RandomPage() {
        const [randomPage, { refetch }] = createResource(async () => {
            if (!import.meta.env.SSR) {
                return fetchRandomPage(props.wiki_name!);
            } else {
                return undefined;
            }
        });

        let tooltipRef!: HTMLButtonElement;

        onMount(async () => {
            if (props.wiki_name !== undefined) {
                refetch();
            }

            await reinitializeFlowBiteTooltips(tooltipRef);
        });

        return (
            <div class="px-2 py-2 flex flex-col">
                <div class="flex justify-between">
                    <p>Try:</p>
                    {props.wiki_name !== undefined && (
                        <div>
                            <TooltipButton
                                class="button dark-layer-3 px-1 py-1 group"
                                type="button"
                                onClick={async () => {
                                    setShow(true);
                                    refetch();
                                }}
                                tooltip="Try another">
                                <span class="w-6 h-6 block">
                                    <Refresh />
                                </span>
                            </TooltipButton>
                        </div>
                    )}
                </div>
                <ErrorBoundary
                    fallback={
                        <p class="error text-sm">
                            Failed fetching random page from https://
                            {get_wiki_prefix(props.wiki_name!)}
                            .wikipedia.org/api/rest_v1/page/random/title
                        </p>
                    }>
                    <Suspense
                        fallback={<p class="my-1 w-36 h-6 bg-skeleton" />}>
                        <div class="flex flex-wrap gap-2">
                            <p
                                class="break-all hover:underline cursor-pointer"
                                onClick={() => {
                                    setValue(randomPage());
                                    updateSuggestions(randomPage()!);
                                }}>
                                {randomPage()}
                            </p>
                            <Show
                                when={
                                    randomPage() !== undefined &&
                                    props.wiki_name !== undefined
                                }>
                                <PageLinkPills
                                    title={randomPage()}
                                    wiki_name={props.wiki_name!}
                                />
                            </Show>
                        </div>
                    </Suspense>
                </ErrorBoundary>
            </div>
        );
    }

    const selectIdName = (index: number): string => {
        return `page-search-${componentID}-${index}`;
    };

    const handleKey = (
        data: { page_title: any; index: number },
        e: KeyboardEvent & {
            currentTarget: HTMLDivElement;
            target: Element;
        },
    ) => {
        if (e.key == "Enter") {
            setValue(data.page_title);
        } else if (e.key == "ArrowDown") {
            e.preventDefault(); // prevent autoscrolling
            document.getElementById(selectIdName(data.index + 1))?.focus();
        } else if (e.key == "ArrowUp") {
            e.preventDefault(); // prevent autoscrolling
            if (data.index == 0) {
                inputRef?.focus();
            }
            document.getElementById(selectIdName(data.index - 1))?.focus();
        }
    };

    return (
        <div class={clsx("relative h-full", props.classNameWrapper)}>
            <FloatingLabelInput
                autocomplete="off"
                label={props.placeholder}
                type="text"
                name={props.name}
                // placeholder={props.placeholder}
                value={value()?.replaceAll(" ", "_") ?? ""}
                class="input-default dark-layer-1 px-2 py-1 w-full h-full"
                disabled={props.wiki_name === undefined}
                onInput={(e) => {
                    updateSuggestions(e.currentTarget.value); // element might have focus before js was registered?
                    setShow(true);
                    setValue(e.currentTarget.value);
                }}
                onFocus={() => {
                    setShow(true);
                }}
                clickOutside={() => {
                    setShow(false);
                }}
                ref={inputRef}
                onKeyDown={(e) => {
                    if (e.key == "ArrowDown") {
                        // Focus first select div
                        e.preventDefault(); // prevent autoscrolling
                        document.getElementById(selectIdName(0))?.focus();
                    }
                }}
            />
            {/* <div class="border h-full w-full p-3 relative">aaa</div> */}

            <div
                class={clsx(
                    "absolute bg-white border [:not(.dark)]border-neutral-200 dark-layer-2 w-full mt-1.5 z-20 " +
                        "min-h-10 max-h-[380px]",
                    !show() && "hidden border-none",
                    value()?.length != 0 && "overflow-y-scroll overflow-x-clip",
                )}>
                {loading() && (
                    <div class="absolute backdrop-blur-[1px] dark:bg-dark_02/10 bg-gray-300/10 w-full h-full flex justify-center">
                        <div
                            class={clsx(
                                pages().length > 2
                                    ? "absolute top-[15%]"
                                    : "flex items-center",
                            )}>
                            <span class="block w-8 h-8">
                                <LoadingSpinner />
                            </span>
                        </div>
                    </div>
                )}

                {/* {loading() && <p class="px-2 py-1">loading...</p>} */}
                <Show
                    when={
                        pages() &&
                        pages().length == 0 &&
                        !loading() &&
                        value()?.length != 0
                    }>
                    <p class="px-2 py-1">No results found</p>
                </Show>

                <Show when={error() !== undefined}>
                    <p class="text-red-400 text-base">{error()}</p>
                </Show>

                <For each={pages()}>
                    {(page, index) => (
                        <div
                            class={clsx(
                                "px-2 py-1 first-letter:break-all cursor-pointer",
                                "hover:bg-gray-100 dark:hover:!bg-dark_03 focus:bg-gray-100 dark:focus:bg-dark_03 focus:outline-none",
                                page.title == value() &&
                                    "bg-gray-200 dark:bg-dark_04",
                            )}
                            onClick={() => setValue(page.title)}
                            onKeyDown={[
                                handleKey,
                                { page_title: page.title, index: index() },
                            ]}
                            tabindex="0"
                            id={selectIdName(index())}>
                            <div class="flex flex-wrap justify-between items-center gap-1">
                                <p class="break-words break-normal leading-tight">
                                    {page.title}
                                </p>
                                <PageLinkPills
                                    title={page.title}
                                    wiki_name={props.wiki_name!}
                                />
                            </div>

                            <p class="text-secondary text-sm break-words break-normal">
                                {page.description}
                            </p>
                        </div>
                    )}
                </For>
                {value()?.length == 0 && <RandomPage />}
            </div>
        </div>
    );
}

export default PageSearch;
