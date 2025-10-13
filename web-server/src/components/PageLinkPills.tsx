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

    const [num_links, { refetch: refetch_outgoing }] = createResource(
        async () => {
            // await sleep(1000);
            const url = `${import.meta.env.SITE}/api/${props.wiki_name}/${props.title.replaceAll(
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
            const url = `${import.meta.env.SITE}/api/${props.wiki_name}/${props.title.replaceAll(
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
                    }>
                    <TooltipButton
                        class="w-5 h-5 block -mb-0.5"
                        tooltip="Incoming Links"
                        type="button">
                        <IncomingLinks />
                    </TooltipButton>
                   {big_num_format(times_linked())}
                </ErrorBoundary>
            </Pill>

            <Pill aria-labelledby="outgoing-tip">
                <ErrorBoundary
                    fallback={
                        <FallBack error_msg={num_links.error.toString()} />
                    }>
                    {big_num_format(num_links())}
                    <TooltipButton
                        class="w-5 h-5 block -mb-0.5"
                        tooltip="Outgoing Links"
                        type="button">
                        <OutgoingLinks />
                    </TooltipButton>
                </ErrorBoundary>
            </Pill>
        </div>
    );
}
