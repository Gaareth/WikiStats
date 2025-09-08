import clsx from "clsx";
import { For, Suspense, createResource, createSignal } from "solid-js";
import type { PageLinks } from "../pages/api/[wiki_name]/[...page_name]/links";
import { wiki_link } from "../utils";
import { LoadingSpinner } from "./ClientIcons/Icons";
import { PathLink } from "./SP/PathLink";

const PAGINATION_SIZE = 50;

const fetchLinks = async ({
    wiki_name,
    page_name,
    incoming,
    page = 1,
}: Props & { page?: number }): Promise<PageLinks | undefined> => {
    //   await sleep(4000);
    const endpoint = incoming ? "linked" : "links";
    const resp = await fetch(
        `/api/${wiki_name}/${page_name}/${endpoint}?page=${page}`,
    );
    if (resp.ok) {
        return await resp.json();
    }
};

interface Props {
    wiki_name: string;
    page_name: string;
    incoming?: boolean;
    total: number;
}

function Links(props: Props) {
    const [links, { mutate }] = createResource(props, fetchLinks);
    const [page, setPage] = createSignal(1);
    const [paginationLoading, setPaginationLoading] = createSignal(false);

    const loadMore = async () => {
        setPaginationLoading(true);
        setPage(page() + 1);
        let res = await fetchLinks({ ...props, page: page() });
        console.log(res);

        // @ts-ignore - works lol
        mutate([...links(), ...res]);
        setPaginationLoading(false);
    };

    return (
        <>
            <section>
                <h1 class="text-2xl my-1" id="links">
                    {props.incoming
                        ? "List of Pages linking here"
                        : "List of linked Pages"}
                </h1>

                <Suspense fallback={<div>loading...</div>}>
                    {links.loading && (
                        <div class="flex items-center gap-1">
                            <span class="block w-6 h-6">
                                {" "}
                                <LoadingSpinner />
                            </span>
                            Loading...
                        </div>
                    )}
                    <div class="flex flex-col gap-2">
                        <For each={links()}>
                            {(link, _) => (
                                <li class="list-none">
                                    <div
                                        class={clsx(
                                            "flex items-center gap-3",
                                            !props.incoming &&
                                                "flex-row-reverse justify-end",
                                        )}
                                    >
                                        <a
                                            href={
                                                link.pageTitle &&
                                                wiki_link(
                                                    link.pageTitle,
                                                    props.wiki_name,
                                                )
                                            }
                                            class={clsx(
                                                "break-all w-full sm:w-auto",
                                                link.pageTitle == null &&
                                                    "text-red-400 hover:no-underline",
                                            )}
                                        >
                                            {link.pageTitle ??
                                                "error resolving pageId: " +
                                                    link.pageLink}
                                        </a>
                                        <PathLink
                                            current_page_title={
                                                !props.incoming
                                                    ? props.page_name
                                                    : link.pageTitle!
                                            }
                                            next_page_title={
                                                props.incoming
                                                    ? props.page_name
                                                    : link.pageTitle!
                                            }
                                            iconClassName="-rotate-90 w-4 h-4"
                                        />
                                    </div>
                                </li>
                            )}
                        </For>
                        {(links()?.length ?? 0) < props.total && (
                            <button
                                type="button"
                                onClick={loadMore}
                                class="button dark-layer-1 mt-2"
                            >
                                {paginationLoading() ? "loading" : "load"}{" "}
                                more...
                            </button>
                        )}
                    </div>
                </Suspense>
            </section>
        </>
    );
}

export default Links;
