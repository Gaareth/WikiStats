import { Show, type Accessor } from "solid-js";
import type { streamData } from "./stream";

export default function SPResult(props: { streamData: Accessor<streamData> }) {
    return (
        <>
            <Show
                when={
                    props.streamData()?.paths != null &&
                    props.streamData()?.paths?.length! > 0
                }
            >
                <p class="my-3">
                    Found a{" "}
                    <span class="font-bold">
                        {props.streamData()!.paths![0]?.length}
                    </span>{" "}
                    pages long path in{" "}
                    <span class="font-bold">
                        {props.streamData()?.elapsed_ms}ms
                    </span>{" "}
                    visiting{" "}
                    <span class="font-bold">{props.streamData()?.visited}</span>{" "}
                    pages
                </p>

                {/* <Switch>
          <Match when={props.streamData()?.visited == 1}>
            <p>Easy, almost like you started at the target</p>
          </Match>
          <Match when={props.streamData()?.visited! >= 100}>
            <p>You have discovered a lot of pages</p>
          </Match>
        </Switch>

        <Switch>
          <Match when={props.streamData()?.paths![0].length! < 5}>
            <p>That's a short path</p>
          </Match>
          <Match when={props.streamData()?.paths![0].length! == 5}>
            <p>Most of the pages are found in 5 steps</p>
          </Match>
          <Match when={props.streamData()?.paths![0].length! == 5}>
            <p>Most of the pages are found in 5 steps</p>
          </Match>
        </Switch>
         */}
            </Show>
        </>
    );
}
