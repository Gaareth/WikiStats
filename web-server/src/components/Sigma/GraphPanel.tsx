import { children } from "solid-js";
import { twMerge } from "tailwind-merge";

interface PanelProps {
    title: string;
    deployed?: boolean;
    children?: any;
    class?: string;
}

export default function GraphPanel(props: PanelProps) {
    const childComponents = children(() => props.children);

    return (
        <details
            class={twMerge(
                "border border-slate-200 dark:border-dark_02 bg-white" +
                    " px-4 py-2 cursor-pointer" +
                    " dark:backdrop-blur-xl dark:bg-dark_01/80 dark:border-dark_02/75",
                props.class,
            )}
        >
            <summary>{props.title} </summary>
            <div class="pt-2 cursor-default">{childComponents()}</div>
        </details>
    );
}
