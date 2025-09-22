import { createResource, Show } from "solid-js";
import { LoadingSpinner } from "./ClientIcons/Icons";

const fetchStatus = async () => {
    const response = await fetch(`api/is-updating`);
    const json = await response.json();    
    return json.is_updating;
};

export default function DBUpdating() {
    const [isUpdating] = createResource(fetchStatus);

    return (
        <Show when={isUpdating()} fallback={null}>
            <span class="inline-flex items-center gap-1 font-bold uppercase">
                <span class="inline-block w-4">
                    <LoadingSpinner />
                </span>
                DATABASE is updating!
                <a href="/dump-progress">More here</a>
            </span>
        </Show>
    );
}
