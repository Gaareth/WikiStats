import { type Component, type JSX, splitProps } from "solid-js";
import { Dynamic } from "solid-js/web";
import { twMerge } from "tailwind-merge";

interface Props {
    className?: string;
    children: JSX.Element;
    wrapperComponent?: keyof JSX.IntrinsicElements | Component<any>;
    [key: string]: any;
}

const Card = (props: Props) => {
    const [local, rest] = splitProps(props, [
        "wrapperComponent",
        "children",
        "className",
    ]);

    return (
        <Dynamic
            component={local.wrapperComponent ?? "div"}
            class={twMerge(
                `py-2 px-3 flex flex-col justify-center border text-center max-w-full
                dark:bg-dark_01 dark:border-dark_01 dark:hover:border-dark_02 dark:border-2 dark:rounded-sm`,
                local.className,
            )}
            {...rest}>
            {local.children}
        </Dynamic>
    );
};

export default Card;
