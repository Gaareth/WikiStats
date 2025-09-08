import { createUniqueId, splitProps, type JSX } from "solid-js";
import { cn } from "../utils";
import clickOutside from "./click-outside";

declare module "solid-js" {
    namespace JSX {
        interface DirectiveFunctions {
            // use:clickOutside
            clickOutside: typeof clickOutside;
        }
    }
}

interface Props {
    label: string;
    class: string;
    clickOutside: () => void;
}

export default function Input(props: JSX.IntrinsicElements["input"] & Props) {
    const [local, rest] = splitProps(props, [
        "label",
        "class",
        "placeholder",
        "clickOutside",
    ]);
    const uuid = createUniqueId();

    return (
        <div class="relative">
            <input
                class={cn(local.class, "peer py-4 disabled:cursor-not-allowed")}
                placeholder=" "
                id={"floating_outlined+" + uuid}
                use:clickOutside={local.clickOutside}
                {...rest}
            />
            <label
                for={"floating_outlined+" + uuid}
                class="bg-white dark:bg-transparent dark:peer-focus:bg-gradient-to-b from-transparent to-5%  from-45% to-dark_01 
                absolute text-base text-secondary duration-300 transform -translate-y-[1.15rem] scale-75 top-2 z-10 origin-[0]  px-2 
                peer-focus:px-2 peer-focus:text-blue-600 peer-focus:dark:text-slate-50 
                peer-placeholder-shown:scale-100 peer-placeholder-shown:-translate-y-1/2 peer-placeholder-shown:top-1/2 
                peer-focus:top-2 peer-focus:scale-75 
                peer-focus:-translate-y-5 rtl:peer-focus:translate-x-1/4 rtl:peer-focus:left-auto start-1 peer-disabled:cursor-not-allowed"
            >
                {local.label}
            </label>
        </div>
    );
}
