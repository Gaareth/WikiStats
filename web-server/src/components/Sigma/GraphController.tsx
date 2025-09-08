import {
    onMount,
    Show,
    splitProps,
    type Accessor,
    type Component,
    type JSX,
} from "solid-js";
import { reinitializeFlowBiteTooltips } from "../../utils";
import { LoadingSpinner } from "../ClientIcons/Icons";
import { TooltipButton } from "../TooltipButton";
import { useSigma } from "./SigmaContext";

export interface ControllerProps {
    resetZoom: (() => void) | false | undefined;
    toggleLayout: (() => void) | false | undefined;
    toggleFullScreen?: (() => void) | false | undefined;
    layoutIsRunning: Accessor<boolean>;
}

export default function GraphController(props: ControllerProps) {
    const playIcon = () => (
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width="1em"
            height="1em"
            viewBox="0 0 24 24"
            class="w-6 h-6 hover:text-blue-500"
        >
            <path fill="currentColor" d="M8 5.14v14l11-7z" />
        </svg>
    );
    const pauseIcon = () => (
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width="1em"
            height="1em"
            viewBox="0 0 24 24"
            class="w-6 h-6 hover:text-blue-500"
        >
            <path fill="currentColor" d="M14 19h4V5h-4M6 19h4V5H6z" />
        </svg>
    );

    const fullScreenIcon = () => (
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width="1em"
            height="1em"
            viewBox="0 0 24 24"
            class="w-6 h-6 hover:text-blue-500"
        >
            <path
                fill="currentColor"
                d="M5 5h5v2H7v3H5zm9 0h5v5h-2V7h-3zm3 9h2v5h-5v-2h3zm-7 3v2H5v-5h2v3z"
            />
        </svg>
    );

    const resetIcon = () => (
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width="1em"
            height="1em"
            viewBox="0 0 24 24"
            class="w-6 h-6 hover:text-blue-500"
            aria-label="arrow looping around dot, starting in the west ending nord so that the head points west"
        >
            <path
                fill="currentColor"
                d="M12 16c1.671 0 3-1.331 3-3s-1.329-3-3-3s-3 1.331-3 3s1.329 3 3 3"
            />
            <path
                fill="currentColor"
                d="M20.817 11.186a8.94 8.94 0 0 0-1.355-3.219a9.053 9.053 0 0 0-2.43-2.43a8.95 8.95 0 0 0-3.219-1.355a9.028 9.028 0 0 0-1.838-.18V2L8 5l3.975 3V6.002c.484-.002.968.044 1.435.14a6.961 6.961 0 0 1 2.502 1.053a7.005 7.005 0 0 1 1.892 1.892A6.967 6.967 0 0 1 19 13a7.032 7.032 0 0 1-.55 2.725a7.11 7.11 0 0 1-.644 1.188a7.2 7.2 0 0 1-.858 1.039a7.028 7.028 0 0 1-3.536 1.907a7.13 7.13 0 0 1-2.822 0a6.961 6.961 0 0 1-2.503-1.054a7.002 7.002 0 0 1-1.89-1.89A6.996 6.996 0 0 1 5 13H3a9.02 9.02 0 0 0 1.539 5.034a9.096 9.096 0 0 0 2.428 2.428A8.95 8.95 0 0 0 12 22a9.09 9.09 0 0 0 1.814-.183a9.014 9.014 0 0 0 3.218-1.355a8.886 8.886 0 0 0 1.331-1.099a9.228 9.228 0 0 0 1.1-1.332A8.952 8.952 0 0 0 21 13a9.09 9.09 0 0 0-.183-1.814"
            />{" "}
        </svg>
    );

    const { loaded } = useSigma()!["graph"];

    interface ControlButtonProps {
        tooltip: string;
        onClick: JSX.EventHandler<HTMLButtonElement, MouseEvent>;
        children?: JSX.Element;
    }

    const ControlButton: Component<ControlButtonProps> = (props) => {
        const [local, rest] = splitProps(props, ["children"]);

        return (
            <TooltipButton class="dark-layer-1 border-0 px-0 group" {...rest}>
                {local.children}
            </TooltipButton>
        );
    };

    let resetZoomRef!: HTMLButtonElement;
    onMount(async () => {
        await reinitializeFlowBiteTooltips(resetZoomRef);
    });

    return (
        <div>
            <div class="w-fit mx-auto flex flex-row gap-3 border h-10 px-2 bg-white dark:bg-dark_01 dark:border-dark_02">
                {props.resetZoom && (
                    <ControlButton
                        onClick={props.resetZoom}
                        tooltip="Reset zoom"
                    >
                        <p class="hidden">
                            Attribution: BoxIcon bx:reset
                            https://icon-sets.iconify.design/bx/reset/
                        </p>
                        {resetIcon()}
                    </ControlButton>
                )}
                {props.toggleLayout && (
                    <ControlButton
                        onClick={props.toggleLayout}
                        tooltip={`${props.layoutIsRunning() ? "Stop" : "Start"} layouting nodes`}
                    >
                        {props.layoutIsRunning() ? pauseIcon() : playIcon()}
                    </ControlButton>
                )}
                {props.toggleFullScreen && (
                    <ControlButton
                        onClick={props.toggleFullScreen}
                        tooltip="Enter/leave fullscreen"
                    >
                        {fullScreenIcon()}
                    </ControlButton>
                )}
            </div>
            <Show when={!loaded()}>
                <div class="flex items-center gap-1 mt-0.5">
                    <p class="text-base text-secondary text-center animate-pulse">
                        loading nodes
                    </p>
                    <span class="block w-4">
                        <LoadingSpinner />
                    </span>
                </div>
            </Show>
        </div>
    );
}
