import { splitProps, Suspense } from "solid-js";
import { twMerge } from "tailwind-merge";

export default function Pill(props: { children: any; class?: string }) {
    const [_, rest] = splitProps(props, ["children", "class"]);
    return (
        <Suspense
            fallback={<div class="bg-skeleton rounded w-9 h-[1.125rem]" />}>
            <div
                class={twMerge(
                    "bg-skeleton-no-pulse dark:text-white rounded text-sm px-1 py-[0.05rem] flex items-center tooltip-wrapper group",
                    props.class,
                )}
                {...rest}>
                {props.children}
            </div>
        </Suspense>
    );
}
