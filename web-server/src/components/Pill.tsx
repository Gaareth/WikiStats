import { splitProps, Suspense } from "solid-js";
import { Dynamic } from "solid-js/web";
import { twMerge } from "tailwind-merge";

export default function Pill(props: {
    children: any;
    class?: string;
    link?: string;
}) {
    const [_, rest] = splitProps(props, ["children", "class", "link"]);
    return (
        <Suspense
            fallback={<div class="bg-skeleton rounded w-9 h-[1.125rem]" />}>
            <Dynamic
                component={props.link != null ? "a" : "div"}
                href={props.link}
                class={twMerge(
                    "bg-skeleton-no-pulse dark:text-white rounded text-sm px-1 py-[0.05rem] flex items-center tooltip-wrapper group",
                    props.class,
                )}
                {...rest}>
                {props.children}
            </Dynamic>
        </Suspense>
    );
}
