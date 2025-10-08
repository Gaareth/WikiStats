import {
    createUniqueId,
    onMount,
    splitProps,
    type Component,
    type JSX,
} from "solid-js";
import { reinitializeFlowBiteTooltips } from "../utils";

interface TooltipButtonProps {
    tooltip: string;
    onClick?: JSX.EventHandler<HTMLButtonElement, MouseEvent>;
    disabled?: boolean;
    class?: string;
    type?: "button" | "submit" | "reset";
    children?: JSX.Element;
}

export const TooltipButton: Component<TooltipButtonProps> = (props) => {
    const [local, rest] = splitProps(props, ["children", "tooltip"]);
    const componentID = createUniqueId();

    let btnRef!: HTMLButtonElement;
    onMount(async () => {
        await reinitializeFlowBiteTooltips(btnRef);
        btnRef.title = ""; // if javascript is enabled, remove the title to avoid double tooltips
    });

    return (
        <>
            <button
                {...rest}
                aria-labelledby={"tooltip-" + componentID}
                data-tooltip-target={"tooltip-" + componentID}
                ref={btnRef}
                title={local.tooltip}
            >
                {local.children}
            </button>

            <div
                id={"tooltip-" + componentID}
                class="tooltip default-tooltip"
                role="tooltip">
                {local.tooltip}
                <div class="tooltip-arrow" data-popper-arrow />
            </div>
        </>
    );
};
