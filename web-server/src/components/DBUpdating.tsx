import { createResource, Show } from "solid-js";
import {
    LoadingSpinner,
    MaterialSymbolsConstructionRounded,
} from "./ClientIcons/Icons";
import { TooltipButton } from "./TooltipButton";

const fetchStatus = async () => {
    const response = await fetch(`api/server-status`);
    const json: {
        is_updating: boolean;
        is_rebuilding: boolean;
    } = await response.json();
    return json;
};

export default function DBUpdating() {
    const [status] = createResource(fetchStatus);

    return (
        <div class="flex flex-wrap gap-1 sm:gap-5 justify-center">
            <Show when={status()?.is_updating} fallback={null}>
                <span class="inline-flex items-center gap-1 font-bold uppercase">
                    <span class="inline-block w-4">
                        <LoadingSpinner />
                    </span>
                    DATABASE is updating!
                    <a href="/dump-progress">More here</a>
                </span>
            </Show>

            <Show when={status()?.is_rebuilding} fallback={null}>
                <TooltipButton tooltip="The server is rebuilding its pages. New stats will be available when this is complete.">
                    <span class="inline-flex items-center gap-1 font-bold uppercase text-orange-500 animate-pulse">
                        <span class="inline-block w-4">
                            <MaterialSymbolsConstructionRounded />
                        </span>
                        rebuilding
                    </span>
                </TooltipButton>
            </Show>
        </div>
    );
}
