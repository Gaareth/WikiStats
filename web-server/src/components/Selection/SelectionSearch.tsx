import clsx from "clsx";
import { createSignal, createUniqueId, onMount, Show } from "solid-js";
import { cn } from "../../utils";
import clickOutside from "../click-outside";

declare module "solid-js" {
    namespace JSX {
        interface DirectiveFunctions {
            // use:clickOutside
            clickOutside: typeof clickOutside;
        }
    }
}

interface Props {
    items: { name: string; url: string }[];
    selected?: string;
}

export default function SearchWithDropdown(props: Props) {
    const [suggestions, setSuggestions] = createSignal(props.items);
    const [isDropdownVisible, setDropdownVisible] = createSignal(false);

    const onInput = (e: { target: { value: any } }) => {
        setSuggestions(
            props.items.filter((item) => item.name.startsWith(e.target.value)),
        );
        setDropdownVisible(true);
    };

    const uuid = createUniqueId();

    // disable suggestions when javascript is available
    let input!: HTMLInputElement;
    onMount(() => (input.autocomplete = "off"));


    return (
        <form
            action="/api/selection-search-redirect"
            method="get"
            class="relative">
            <input
                type="text"
                placeholder="Select more"
                class="ml-auto dark-layer-2 dark:text-slate-50 input-default dark-layer-2 w-full"
                list={"datalist-selection-" + uuid}
                name="resource"
                ref={input}
                // value={query()}
                onInput={onInput}
                onFocus={() => setDropdownVisible(true)}
                use:clickOutside={() => setDropdownVisible(false)}
                required
            />

            <noscript>
                <datalist id={"datalist-selection-" + uuid}>
                    {props.items.map((item) => (
                        <option value={item.url}>{item.name}</option>
                    ))}
                </datalist>
            </noscript>
            <noscript>
                <p class="text-sm text-error">
                    Because you have javascript disabled: Select, press enter,
                    get redirected
                </p>
            </noscript>

            <Show when={isDropdownVisible()}>
                <div
                    class={clsx(
                        "absolute bg-white border [:not(.dark)]border-neutral-200 dark-layer-1 w-full z-20 " +
                            "min-h-10 max-h-[380px] overflow-y-scroll overflow-x-clip",
                    )}>
                    {suggestions().map((item) => (
                        <a
                            href={item.url}
                            class={cn(
                                "px-2 hover:dark:bg-dark_02 w-full block",
                                item.name === props.selected && "font-bold",
                            )}>
                            {item.name}
                        </a>
                    ))}
                </div>
            </Show>
        </form>
    );
}
