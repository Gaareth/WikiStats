import { createSignal } from "solid-js";

interface Props {
    text?: string;
    // children: JSX.Element;
}

export const ShowMore = (props: Props) => {
    const [show, setShow] = createSignal(false);

    return (
        <p
            onClick={(e) => e.target.classList.toggle("line-clamp-1")}
            class="line-clamp-1 break-all"
        >
            {props.text}
        </p>
    );
};

export default ShowMore;
// #<div class="grid grid-rows-2 sm:grid-rows-1 grid-flow-col-dense gap-1 w-full">
// <div
//     class={cn(
//         "overflow-hidden text-ellipsis ",
//         show() && "break-all",
//     )}
//     ref={contentRef}
// >
//     {props.children}
// </div>
// <Show when={isClamped() || props.showMore}>
//     <button onClick={() => setShow(!show())} class="button inline-block">
//         Show {show() ? "less" : "more"}
//     </button>
// </Show>
// </div>
