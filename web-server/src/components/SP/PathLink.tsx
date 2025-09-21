import { createUniqueId, onMount } from "solid-js";
import { twMerge } from "tailwind-merge";
import { reinitializeFlowBiteTooltips } from "../../utils";
import { MaterialSymbolsArrowDownwardRounded } from "../ClientIcons/Icons";

export const PathLink = (props: {
    current_page_title: string;
    next_page_title: string;
    className?: string;
    iconClassName?: string;
}) => {
    const id = createUniqueId();
    let tooltipRef!: HTMLAnchorElement;

    onMount(async () => {
        await reinitializeFlowBiteTooltips(tooltipRef);
    });

    return (
        <div class={twMerge("flex", props.className)}>
            <a
                class="border rounded-full p-1 bg-white hover:bg-neutral-50 
          dark-layer-2  hover:dark:!bg-dark_03 group z-10"
                href={`mirror/${props.current_page_title}?link=${props.next_page_title}#scroll`}
                data-tooltip-target={"tooltip-path-link-" + id}
                aria-labelledby={"tooltip-path-link-" + id}
                ref={tooltipRef}>
                <span class={twMerge("block w-5", props.iconClassName)}>
                    <MaterialSymbolsArrowDownwardRounded aria-label="right-pointing arrow" />
                </span>
            </a>
            <div
                class="tooltip default-tooltip z-20"
                id={"tooltip-path-link-" + id}
                role="tooltip">
                Show me where this link is on the wikipedia page
                <div class="tooltip-arrow" data-popper-arrow />
            </div>
        </div>
    );
};
