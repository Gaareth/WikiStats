import { onMount } from "solid-js";
import { cn, get_wiki_prefix, wiki_link } from "../utils";
import Pill from "./Pill";

interface Props {
    wiki_name: string;
    page_name: string;
    class_name?: string;
}

const WikiLink = (props: Props) => {
    let linkRef!: HTMLAnchorElement;
    const maxLines = 2;
    const lineClampClass = `line-clamp-${maxLines}`;

    // Always add the line clamp class initially, because if no javascript, we want it to clamp
    // If javascript is enabled, we will remove it if needed on mount
    props.class_name = cn(props.class_name, lineClampClass);
    onMount(() => {
        if (linkRef) {
            const lineHeight = parseFloat(getComputedStyle(linkRef).lineHeight);
            const maxHeight = lineHeight * maxLines;
            if (linkRef.scrollHeight <= maxHeight + 1) {
                linkRef.classList.remove(lineClampClass);
            }
        }
    });

    return (
        <div class="flex gap-1 sm:gap-2 items-center overflow-hidden text-base sm:text-xl text-left">
            <a href={wiki_link(props.page_name, props.wiki_name)}>
                <Pill class="w-6 flex justify-center text-blue-700">
                    {get_wiki_prefix(props.wiki_name)}
                </Pill>
            </a>

            <a
                ref={linkRef}
                href={`/wiki/${props.wiki_name}/${props.page_name}`}
                class={cn(
                    lineClampClass,
                    "break-words text-ellipsis overflow-hidden text-center",
                    props.class_name,
                )}
                onClick={(e) => {
                    if (e.target.classList.contains(lineClampClass)) {
                        e.preventDefault();
                        e.target.classList.remove(lineClampClass);
                    }
                }}>
                {props.page_name.replaceAll("_", " ")}
            </a>
        </div>
    );
};

export default WikiLink;
