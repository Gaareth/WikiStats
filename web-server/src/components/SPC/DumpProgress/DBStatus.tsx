import { Show } from "solid-js";
import { cn } from "../../../utils";
import {
    ErrorCircleIcon,
    LoadingSpinner,
    MaterialSymbolsCalendarClock,
    MaterialSymbolsDoneAllRounded,
} from "../../ClientIcons/Icons";

type DBStatusProps = {
    dbStatus: "RUNNING" | "DONE" | "SCHEDULED" | "FAILED";
};

export default function DBStatus(props: DBStatusProps) {
    return (
        <div
            class={cn(
                "ml-auto flex gap-2 items-center dark-layer-2 rounded px-3 border uppercase",
                props.dbStatus === "DONE" &&
                    "dark:border-green-300/20 border-green-400",
                props.dbStatus === "FAILED" &&
                    "dark:border-red-400/20 border-red-500",
            )}>
            <Show when={props.dbStatus === "RUNNING"}>
                IN PROGRESS
                <span class="block w-4">
                    <LoadingSpinner />
                </span>
            </Show>
            <Show when={props.dbStatus === "DONE"}>
                DONE
                <span class="block w-5 text-green-400 dark:text-green-300">
                    <MaterialSymbolsDoneAllRounded aria-label="two checkmarks" />
                </span>
            </Show>
            <Show when={props.dbStatus === "SCHEDULED"}>
                SCHEDULED
                <span class="block w-5">
                    <MaterialSymbolsCalendarClock aria-label="schedule" />
                </span>
            </Show>
            <Show when={props.dbStatus === "FAILED"}>
                FAILED
                <span class="block w-5 text-red-500 dark:text-red-400">
                    <ErrorCircleIcon aria-label="Failure" />
                </span>
            </Show>
        </div>
    );
}
