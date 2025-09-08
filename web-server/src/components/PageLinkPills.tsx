import {
    createEffect,
    createResource,
    createUniqueId,
    ErrorBoundary,
    onMount,
    Suspense,
} from "solid-js";
import { big_num_format, reinitializeFlowBiteTooltips } from "../utils";
import {
    ErrorCircleIcon,
    IncomingLinks,
    OutgoingLinks,
} from "./ClientIcons/Icons";
import Pill from "./Pill";
import { TooltipButton } from "./TooltipButton";

export default function PageLinkPills(props: {
    title: string;
    wiki_name: string;
}) {
    const componentID = createUniqueId();
    let incomingRef!: HTMLButtonElement;
    let outgoingRef!: HTMLButtonElement;

    // TODO: hardcoded port oh no pls fix
    const base = import.meta.env.SSR ? "http://localhost:4321" : "";

    const [num_links, { refetch: refetch_outgoing }] = createResource(
        async () => {
            // await sleep(1000);
            const url = `${base}/api/${props.wiki_name}/${props.title.replaceAll(
                " ",
                "_",
            )}/links?num=true`;
            const resp = await fetch(url);

            if (!resp.ok) {
                throw Error(url + " : " + resp.statusText);
            }

            return await resp.json();
        },
    );

    const [times_linked, { refetch: refetch_incoming }] = createResource(
        async () => {
            const url = `${base}/api/${props.wiki_name}/${props.title.replaceAll(
                " ",
                "_",
            )}/linked?num=true`;
            const resp = await fetch(url);
            if (!resp.ok) {
                throw Error(
                    "Failed fetching: " + url + " : " + resp.statusText,
                );
            }

            return await resp.json();
        },
    );

    createEffect(() => {
        refetch_outgoing();
        refetch_incoming();

        props.title;
    });

    onMount(async () => {
        await reinitializeFlowBiteTooltips(incomingRef);
        await reinitializeFlowBiteTooltips(outgoingRef);
    });

    const FallBack = (props: { error_msg: string }) => (
        <TooltipButton tooltip={props.error_msg}>
            <span class="w-5 h-5 block error">
                <ErrorCircleIcon />
            </span>
        </TooltipButton>
    );

    return (
        <div class="flex gap-1 items-center">
            <Pill aria-labelledby="incoming-tip">
                <ErrorBoundary
                    fallback={
                        <FallBack error_msg={times_linked.error.toString()} />
                    }
                >
                    <button
                        class="w-5 h-5 block -mb-0.5"
                        aria-labelledby={"tooltip-incoming-" + componentID}
                        data-tooltip-target={"tooltip-incoming-" + componentID}
                        ref={incomingRef}
                        type="button"
                    >
                        <IncomingLinks />
                    </button>
                    <div
                        id={"tooltip-incoming-" + componentID}
                        class="tooltip default-tooltip"
                        role="tooltip"
                    >
                        Incoming Links
                        <div class="tooltip-arrow" data-popper-arrow />
                    </div>
                    <Suspense>{big_num_format(times_linked())}</Suspense>
                </ErrorBoundary>
            </Pill>

            <Pill aria-labelledby="outgoing-tip">
                <ErrorBoundary
                    fallback={
                        <FallBack error_msg={num_links.error.toString()} />
                    }
                >
                    <Suspense>{big_num_format(num_links())}</Suspense>
                    <button
                        class="w-5 h-5 block -mb-0.5"
                        aria-labelledby={"tooltip-outgoing-" + componentID}
                        data-tooltip-target={"tooltip-outgoing-" + componentID}
                        ref={outgoingRef}
                        type="button"
                    >
                        <OutgoingLinks />
                    </button>
                    <div
                        id={"tooltip-outgoing-" + componentID}
                        class="tooltip default-tooltip"
                        role="tooltip"
                    >
                        Outgoing Links
                        <div class="tooltip-arrow" data-popper-arrow />
                    </div>
                </ErrorBoundary>
            </Pill>
        </div>
    );
}
