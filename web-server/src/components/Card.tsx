import type { JSX } from "solid-js";
import { twMerge } from "tailwind-merge";

interface Props {
    className?: string;
    children: JSX.Element;
}

const Card = (props: Props) => {
    return (
        <div
            class={twMerge(
                `py-2 px-3 flex flex-col justify-center border text-center max-w-full
                dark:bg-dark_01 dark:border-dark_01 dark:hover:border-dark_02 dark:border-2 dark:rounded-sm`,
                props.className,
            )}
        >
            {props.children}
        </div>
    );
};

export default Card;
