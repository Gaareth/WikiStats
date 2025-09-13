import { Match, Switch } from "solid-js";
import { cn } from "../../../../utils";
import {
    ErrorCircleIcon,
    LoadingSpinner,
    MaterialSymbolsCalendarClock,
    MaterialSymbolsCheckCircleOutline,
    QuestionMarkCircleIcon,
} from "../../../ClientIcons/Icons";
import { TooltipButton } from "../../../TooltipButton";
import type { TaskStatus } from "./Task.astro";

interface Props {
    status: TaskStatus;
}

export function TaskStatusIndicator(props: Props) {
    const { status } = props;

    const statusMsg = {
        DONE: "Task completed successfully",
        FAILED: "Task failed",
        RUNNING: "Task is running",
        QUEUED: "Task is queued",
    };
    return (
        <TooltipButton
            tooltip={status in statusMsg ? statusMsg[status] : "unknown"}>
            <span
                class={cn(
                    "block min-w-5 mt-0.5",
                    status === "DONE" && "text-green-500 dark:text-green-400",
                    status === "FAILED" && "text-red-500 dark:text-red-400",
                    status !== "DONE" && status !== "FAILED" && "text-gray-500",
                )}>
                <Switch
                    fallback={
                        <QuestionMarkCircleIcon aria-label="Unknown status" />
                    }>
                    <Match when={status === "DONE"}>
                        <MaterialSymbolsCheckCircleOutline aria-label="Success" />
                    </Match>
                    <Match when={status === "FAILED"}>
                        <ErrorCircleIcon aria-label="Failure" />
                    </Match>
                    <Match when={status === "RUNNING"}>
                        <LoadingSpinner />
                    </Match>
                    <Match when={status === "QUEUED"}>
                         <MaterialSymbolsCalendarClock aria-label="Schedule" />
                    </Match>
                </Switch>
            </span>
        </TooltipButton>
    );
}
