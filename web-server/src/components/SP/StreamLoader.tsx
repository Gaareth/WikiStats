import type { Accessor, Setter } from "solid-js";
import { big_num_format } from "../../utils";
import { LoadingSpinner } from "../ClientIcons/Icons";
import type { streamData } from "./stream";

interface Props {
    streamData: Accessor<streamData>;
    setStreamData: Setter<streamData>;
}

export default function StreamLoader(props: Props) {
    return (
        <div class="flex flex-col gap-2 !animate-none items-center">
            <span class="block w-6 h-6">
                <LoadingSpinner />
            </span>
            <div class="flex flex-col leading-tight text-base">
                <span>
                    {big_num_format(props.streamData()?.visited)} Pages visited
                </span>
                <span>Elapsed ms: {props.streamData().elapsed_ms}</span>
            </div>

            {/* <span>{big_num_format(Math.round(perSec()))} Pages per second</span>  */}
        </div>
    );
}
