import type { JSX } from "solid-js";

export function LoadingSpinner() {
    return (
        <>
            <span class="hidden">
                Credits: https://github.com/n3r4zzurr0/svg-spinners
            </span>
            <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                role="status"
                aria-label="Loading spinner (smaller white part moving along the radious of a grey circle)">
                <path
                    fill="currentColor"
                    d="M12,1A11,11,0,1,0,23,12,11,11,0,0,0,12,1Zm0,19a8,8,0,1,1,8-8A8,8,0,0,1,12,20Z"
                    opacity=".25"
                />
                <path
                    fill="currentColor"
                    d="M10.14,1.16a11,11,0,0,0-9,8.92A1.59,1.59,0,0,0,2.46,12,1.52,1.52,0,0,0,4.11,10.7a8,8,0,0,1,6.66-6.61A1.42,1.42,0,0,0,12,2.69h0A1.57,1.57,0,0,0,10.14,1.16Z">
                    <animateTransform
                        attributeName="transform"
                        dur="0.75s"
                        repeatCount="indefinite"
                        type="rotate"
                        values="0 12 12;360 12 12"
                    />
                </path>
            </svg>
        </>
    );
}

export function Refresh() {
    return (
        <>
            <span class="hidden">
                https://api.iconify.design/material-symbols:refresh.svg
            </span>
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                {/* <title>Refresh or reload action icon</title> */}
                <title>Refresh</title>

                <path
                    fill="currentColor"
                    d="M12 20q-3.35 0-5.675-2.325T4 12t2.325-5.675T12 4q1.725 0 3.3.712T18 6.75V4h2v7h-7V9h4.2q-.8-1.4-2.187-2.2T12 6Q9.5 6 7.75 7.75T6 12t1.75 4.25T12 18q1.925 0 3.475-1.1T17.65 14h2.1q-.7 2.65-2.85 4.325T12 20"
                />
            </svg>
        </>
    );
}

export function OutgoingLinks(props: { class?: string }) {
    return (
        <>
            <span class="hidden">
                https://api.iconify.design/material-symbols:arrow-outward-rounded.svg
            </span>
            <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                class={props.class}>
                <title>Arrow</title>
                <path
                    fill="currentColor"
                    d="m16 8.4l-8.9 8.9q-.275.275-.7.275t-.7-.275t-.275-.7t.275-.7L14.6 7H7q-.425 0-.712-.288T6 6t.288-.712T7 5h10q.425 0 .713.288T18 6v10q0 .425-.288.713T17 17t-.712-.288T16 16z"
                />
            </svg>
        </>
    );
}

export function IncomingLinks() {
    return <OutgoingLinks class="rotate-90" />;
}

export function ErrorCircleIcon(
    props: { title?: string } & JSX.IntrinsicElements["svg"],
) {
    return (
        <>
            <span class="hidden">
                https://api.iconify.design/material-symbols:error-circle-rounded-outline-sharp.svg
            </span>
            <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                {...props}>
                <title>{props.title}</title>
                <path
                    fill="currentColor"
                    d="M12 17q.425 0 .713-.288Q13 16.425 13 16t-.287-.713Q12.425 15 12 15t-.712.287Q11 15.575 11 16t.288.712Q11.575 17 12 17Zm0-4q.425 0 .713-.288Q13 12.425 13 12V8q0-.425-.287-.713Q12.425 7 12 7t-.712.287Q11 7.575 11 8v4q0 .425.288.712q.287.288.712.288Zm0 9q-2.075 0-3.9-.788q-1.825-.787-3.175-2.137q-1.35-1.35-2.137-3.175Q2 14.075 2 12t.788-3.9q.787-1.825 2.137-3.175q1.35-1.35 3.175-2.138Q9.925 2 12 2t3.9.787q1.825.788 3.175 2.138q1.35 1.35 2.137 3.175Q22 9.925 22 12t-.788 3.9q-.787 1.825-2.137 3.175q-1.35 1.35-3.175 2.137Q14.075 22 12 22Zm0-2q3.35 0 5.675-2.325Q20 15.35 20 12q0-3.35-2.325-5.675Q15.35 4 12 4Q8.65 4 6.325 6.325Q4 8.65 4 12q0 3.35 2.325 5.675Q8.65 20 12 20Zm0-8Z"
                />
            </svg>
        </>
    );
}

export function QuestionMarkCircleIcon(
    props: { title?: string } & JSX.IntrinsicElements["svg"],
) {
    return (
        <>
            <span class="hidden">
                https://api.iconify.design/material-symbols:help-outline-rounded.svg
                Icon from Material Symbols by Google -
                https://github.com/google/material-design-icons/blob/master/LICENSE
            </span>
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                <path
                    fill="currentColor"
                    d="M11.95 18q.525 0 .888-.363t.362-.887t-.362-.888t-.888-.362t-.887.363t-.363.887t.363.888t.887.362m.05 4q-2.075 0-3.9-.788t-3.175-2.137T2.788 15.9T2 12t.788-3.9t2.137-3.175T8.1 2.788T12 2t3.9.788t3.175 2.137T21.213 8.1T22 12t-.788 3.9t-2.137 3.175t-3.175 2.138T12 22m0-2q3.35 0 5.675-2.325T20 12t-2.325-5.675T12 4T6.325 6.325T4 12t2.325 5.675T12 20m.1-12.3q.625 0 1.088.4t.462 1q0 .55-.337.975t-.763.8q-.575.5-1.012 1.1t-.438 1.35q0 .35.263.588t.612.237q.375 0 .638-.25t.337-.625q.1-.525.45-.937t.75-.788q.575-.55.988-1.2t.412-1.45q0-1.275-1.037-2.087T12.1 6q-.95 0-1.812.4T8.975 7.625q-.175.3-.112.638t.337.512q.35.2.725.125t.625-.425q.275-.375.688-.575t.862-.2"
                />
            </svg>
        </>
    );
}

export function MaterialSymbolsArrowCoolDownRounded(
    props: JSX.IntrinsicElements["svg"],
) {
    return (
        <svg
            xmlns="http://www.w3.org/2000/svg"
            // width="1em"
            // height="1em"
            viewBox="0 0 24 24"
            {...props}>
            <path
                fill="currentColor"
                d="M12 21.9q-.2 0-.375-.075T11.3 21.6l-5.6-5.575q-.275-.275-.275-.7T5.7 14.6q.3-.3.713-.3t.712.3L11 18.5v-6.175q0-.425.288-.712t.712-.288t.713.288t.287.712V18.5l3.9-3.9q.275-.275.688-.275t.712.3q.275.275.275.7t-.275.7L12.7 21.6q-.15.15-.325.225T12 21.9m0-12.575q-.425 0-.712-.287T11 8.325v-1q0-.425.288-.712T12 6.325t.713.288t.287.712v1q0 .425-.288.713T12 9.325m0-5q-.425 0-.712-.287T11 3.325t.288-.712t.712-.288t.713.288t.287.712t-.288.713t-.712.287"
            />
        </svg>
    );
}

export function TargetIcon(
    props: { title: string } & JSX.IntrinsicElements["svg"],
) {
    return (
        <>
            <span hidden>
                https://api.iconify.design/fluent:target-32-filled.svg
            </span>
            <svg
                xmlns="http://www.w3.org/2000/svg"
                {...props}
                viewBox="0 0 32 32">
                <title>{props.title}</title>
                <g fill="currentColor">
                    <path d="M16 13.75a2.25 2.25 0 1 0 0 4.5a2.25 2.25 0 0 0 0-4.5" />
                    <path d="M16 8.75a7.25 7.25 0 1 0 0 14.5a7.25 7.25 0 0 0 0-14.5M11.25 16a4.75 4.75 0 1 1 9.5 0a4.75 4.75 0 0 1-9.5 0" />
                    <path d="M16.001 3.75C9.235 3.75 3.75 9.235 3.75 16.001s5.485 12.252 12.251 12.252S28.253 22.767 28.253 16S22.767 3.75 16 3.75M6.25 16.001c0-5.385 4.366-9.751 9.751-9.751c5.386 0 9.752 4.366 9.752 9.751c0 5.386-4.366 9.752-9.752 9.752c-5.385 0-9.751-4.366-9.751-9.752" />
                </g>
            </svg>
        </>
    );
}

export function TargetHitIcon(
    props: { title: string } & JSX.IntrinsicElements["svg"],
) {
    return (
        <>
            <span hidden>
                https://api.iconify.design/fluent:target-arrow-24-filled.svg
            </span>
            <svg
                xmlns="http://www.w3.org/2000/svg"
                {...props}
                viewBox="0 0 24 24">
                <title>{props.title}</title>
                <path
                    fill="currentColor"
                    d="M21.78 6.78a.75.75 0 0 0-.53-1.28H18.5V2.75a.75.75 0 0 0-1.28-.53l-2.5 2.5a.75.75 0 0 0-.22.53v2.836l-1.982 1.982A2.003 2.003 0 0 0 10 12a2 2 0 1 0 3.932-.518L15.914 9.5h2.836a.75.75 0 0 0 .53-.22zM12 2a10 10 0 0 1 3.424.601l-1.412 1.412q-.094.094-.171.2a8 8 0 1 0 5.947 5.947q.105-.078.2-.172l1.41-1.412A10 10 0 0 1 22 12c0 5.523-4.477 10-10 10S2 17.523 2 12S6.477 2 12 2m0 4q.779.002 1.5.19v1.482l-.414.414l-.049.05A4.005 4.005 0 0 0 8 12a4 4 0 1 0 7.864-1.037l.05-.049l.414-.414h1.483A6 6 0 1 1 12 6"
                />
            </svg>
        </>
    );
}

export function TargetMissedIcon(
    props: { title: string } & JSX.IntrinsicElements["svg"],
) {
    return (
        <>
            <span hidden>
                https://api.iconify.design/fluent:target-dismiss-24-filled.svg
            </span>
            <svg
                xmlns="http://www.w3.org/2000/svg"
                {...props}
                viewBox="0 0 24 24">
                <title>{props.title}</title>

                <path
                    fill="currentColor"
                    d="M4 12a8 8 0 0 0 7.492 7.984a6.5 6.5 0 0 0 1.289 1.986Q12.394 22 12 22C6.477 22 2 17.523 2 12S6.477 2 12 2s10 4.477 10 10q0 .395-.03.78a6.5 6.5 0 0 0-1.986-1.288A8 8 0 0 0 4 12m7.194 3.919a4.001 4.001 0 1 1 4.725-4.725a6.5 6.5 0 0 1 2-.18A6.002 6.002 0 0 0 6 12a6 6 0 0 0 5.013 5.92a6.6 6.6 0 0 1 .18-2.001M12 14h.022A6.5 6.5 0 0 1 14 12.022V12a2 2 0 1 0-2 2m5.5-2a5.5 5.5 0 1 1 0 11a5.5 5.5 0 0 1 0-11m-2.407 2.966l-.07.058l-.057.07a.5.5 0 0 0 0 .568l.058.07l1.77 1.769l-1.768 1.766l-.057.07a.5.5 0 0 0 0 .568l.057.07l.07.057a.5.5 0 0 0 .568 0l.07-.057l1.766-1.767l1.77 1.769l.069.058a.5.5 0 0 0 .568 0l.07-.058l.057-.07a.5.5 0 0 0 0-.568l-.057-.07l-1.77-1.768l1.772-1.77l.058-.069a.5.5 0 0 0 0-.569l-.058-.069l-.069-.058a.5.5 0 0 0-.569 0l-.069.058l-1.772 1.77l-1.77-1.77l-.068-.058a.5.5 0 0 0-.493-.043z"
                />
            </svg>
        </>
    );
}

export function StartIcon(
    props: { title: string } & JSX.IntrinsicElements["svg"],
) {
    return (
        <>
            <span hidden>https://api.iconify.design/ic:round-start.svg</span>
            <svg
                xmlns="http://www.w3.org/2000/svg"
                {...props}
                viewBox="0 0 24 24">
                <title>{props.title}</title>

                <path
                    fill="currentColor"
                    d="M15.29 17.29c.39.39 1.02.39 1.41 0l4.59-4.59a.996.996 0 0 0 0-1.41L16.7 6.7a.996.996 0 0 0-1.41 0c-.38.39-.39 1.03 0 1.42L18.17 11H7c-.55 0-1 .45-1 1s.45 1 1 1h11.17l-2.88 2.88a.996.996 0 0 0 0 1.41M3 18c.55 0 1-.45 1-1V7c0-.55-.45-1-1-1s-1 .45-1 1v10c0 .55.45 1 1 1"
                />
            </svg>
        </>
    );
}

export function MenuIcon(props: JSX.IntrinsicElements["svg"]) {
    return (
        <>
            <span hidden>
                https://api.iconify.design/material-symbols:menu.svg
            </span>
            <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                {...props}>
                <path
                    fill="currentColor"
                    d="M3 18v-2h18v2zm0-5v-2h18v2zm0-5V6h18v2z"
                />
            </svg>
        </>
    );
}

export function IonMdStatsIcon(props: JSX.IntrinsicElements["svg"]) {
    return (
        <>
            <span hidden>https://api.iconify.design/ion:md-stats.svg</span>
            <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 512 512"
                {...props}>
                <path d="M176 64h64v384h-64z" fill="currentColor" />
                <path d="M80 336h64v112H80z" fill="currentColor" />
                <path d="M272 272h64v176h-64z" fill="currentColor" />
                <path d="M368 176h64v272h-64z" fill="currentColor" />
            </svg>
        </>
    );
}

export function MaterialSymbolsLightConversionPathIcon(
    props: JSX.IntrinsicElements["svg"],
) {
    return (
        <>
            <span hidden>
                https://api.iconify.design/material-symbols-light:conversion-path.svg
            </span>
            <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                {...props}>
                <path
                    fill="currentColor"
                    d="M19 20.5q-.898 0-1.587-.562q-.688-.563-.853-1.438H11q-1.458 0-2.479-1.021T7.5 15t1.021-2.479T11 11.5h2q1.031 0 1.766-.735t.734-1.769t-.734-1.764T13 6.5H7.44q-.17.875-.856 1.438T5 8.5q-1.042 0-1.77-.728q-.73-.729-.73-1.77t.73-1.771T5 3.5q.898 0 1.584.563T7.44 5.5H13q1.458 0 2.479 1.021T16.5 9t-1.021 2.479T13 12.5h-2q-1.031 0-1.766.735T8.5 15.004t.734 1.764T11 17.5h5.56q.17-.875.856-1.437T19 15.5q1.042 0 1.77.729q.73.728.73 1.769t-.73 1.771T19 20.5M5 7.5q.617 0 1.059-.441T6.5 6t-.441-1.059T5 4.5t-1.059.441T3.5 6t.441 1.059T5 7.5"
                />
            </svg>
        </>
    );
}

export function GraphIcon(props: JSX.IntrinsicElements["svg"]) {
    return (
        <>
            <span hidden>https://api.iconify.design/ph:graph.svg</span>
            <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 256 256"
                {...props}>
                <path
                    fill="currentColor"
                    d="M200 152a31.84 31.84 0 0 0-19.53 6.68l-23.11-18A31.65 31.65 0 0 0 160 128c0-.74 0-1.48-.08-2.21l13.23-4.41A32 32 0 1 0 168 104c0 .74 0 1.48.08 2.21l-13.23 4.41A32 32 0 0 0 128 96a32.59 32.59 0 0 0-5.27.44L115.89 81A32 32 0 1 0 96 88a32.59 32.59 0 0 0 5.27-.44l6.84 15.4a31.92 31.92 0 0 0-8.57 39.64l-25.71 22.84a32.06 32.06 0 1 0 10.63 12l25.71-22.84a31.91 31.91 0 0 0 37.36-1.24l23.11 18A31.65 31.65 0 0 0 168 184a32 32 0 1 0 32-32m0-64a16 16 0 1 1-16 16a16 16 0 0 1 16-16M80 56a16 16 0 1 1 16 16a16 16 0 0 1-16-16M56 208a16 16 0 1 1 16-16a16 16 0 0 1-16 16m56-80a16 16 0 1 1 16 16a16 16 0 0 1-16-16m88 72a16 16 0 1 1 16-16a16 16 0 0 1-16 16"
                />
            </svg>
        </>
    );
}

export function MaterialSymbolsDatabase(props: JSX.IntrinsicElements["svg"]) {
    return (
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" {...props}>
            <path
                fill="currentColor"
                d="M12 11q3.75 0 6.375-1.175T21 7t-2.625-2.825T12 3T5.625 4.175T3 7t2.625 2.825T12 11m0 2.5q1.025 0 2.563-.213t2.962-.687t2.45-1.237T21 9.5V12q0 1.1-1.025 1.863t-2.45 1.237t-2.962.688T12 16t-2.562-.213t-2.963-.687t-2.45-1.237T3 12V9.5q0 1.1 1.025 1.863t2.45 1.237t2.963.688T12 13.5m0 5q1.025 0 2.563-.213t2.962-.687t2.45-1.237T21 14.5V17q0 1.1-1.025 1.863t-2.45 1.237t-2.962.688T12 21t-2.562-.213t-2.963-.687t-2.45-1.237T3 17v-2.5q0 1.1 1.025 1.863t2.45 1.237t2.963.688T12 18.5"
            />
        </svg>
    );
}

export function ArrowBack(props: JSX.IntrinsicElements["svg"]) {
    return (
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" {...props}>
            <path
                fill="currentColor"
                d="m7.825 13l4.9 4.9q.3.3.288.7t-.313.7q-.3.275-.7.288t-.7-.288l-6.6-6.6q-.15-.15-.213-.325T4.426 12t.063-.375t.212-.325l6.6-6.6q.275-.275.688-.275t.712.275q.3.3.3.713t-.3.712L7.825 11H19q.425 0 .713.288T20 12t-.288.713T19 13z"
            />
        </svg>
    );
}

export function MaterialSymbolsLineEndArrowNotchRounded(
    props: JSX.IntrinsicElements["svg"],
) {
    return (
        <>
            <span hidden>
                https://api.iconify.design/material-symbols:line-end-arrow-notch-rounded.svg
            </span>
            <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                {...props}>
                <path
                    fill="currentColor"
                    d="M12.7 17.925q-.35.2-.625-.062T12 17.25L14.425 13H3q-.425 0-.712-.288T2 12t.288-.712T3 11h11.425L12 6.75q-.2-.35.075-.612t.625-.063l7.975 5.075q.475.3.475.85t-.475.85z"
                />
            </svg>
        </>
    );
}

export function MaterialSymbolsArrowDownwardRounded(
    props: JSX.IntrinsicElements["svg"],
) {
    return (
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" {...props}>
            <path
                fill="currentColor"
                d="M11 16.175V5q0-.425.288-.712T12 4t.713.288T13 5v11.175l4.9-4.9q.3-.3.7-.288t.7.313q.275.3.287.7t-.287.7l-6.6 6.6q-.15.15-.325.213t-.375.062t-.375-.062t-.325-.213l-6.6-6.6q-.275-.275-.275-.687T4.7 11.3q.3-.3.713-.3t.712.3z"
            />
        </svg>
    );
}

export function MaterialSymbolsCheckCircleOutline(
    props: JSX.IntrinsicElements["svg"],
) {
    return (
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" {...props}>
            <path
                fill="currentColor"
                d="m10.6 16.6l7.05-7.05l-1.4-1.4l-5.65 5.65l-2.85-2.85l-1.4 1.4zM12 22q-2.075 0-3.9-.788t-3.175-2.137T2.788 15.9T2 12t.788-3.9t2.137-3.175T8.1 2.788T12 2t3.9.788t3.175 2.137T21.213 8.1T22 12t-.788 3.9t-2.137 3.175t-3.175 2.138T12 22m0-2q3.35 0 5.675-2.325T20 12t-2.325-5.675T12 4T6.325 6.325T4 12t2.325 5.675T12 20m0-8"
            />
        </svg>
    );
}

export function MaterialSymbolsClockLoader40(
    props: JSX.IntrinsicElements["svg"],
) {
    return (
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" {...props}>
            <path
                fill="currentColor"
                d="M12 22q-2.075 0-3.9-.788t-3.175-2.137T2.788 15.9T2 12t.788-3.9t2.137-3.175T8.1 2.788T12 2t3.9.788t3.175 2.137T21.213 8.1T22 12t-.788 3.9t-2.137 3.175t-3.175 2.138T12 22m0-2q1.6 0 3.075-.6t2.6-1.725L12 12V4Q8.65 4 6.325 6.325T4 12t2.325 5.675T12 20"
            />
        </svg>
    );
}

export function MaterialSymbolsScheduleOutline(
    props: JSX.IntrinsicElements["svg"],
) {
    return (
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" {...props}>
            <path
                fill="currentColor"
                d="m15.3 16.7l1.4-1.4l-3.7-3.7V7h-2v5.4zM12 22q-2.075 0-3.9-.788t-3.175-2.137T2.788 15.9T2 12t.788-3.9t2.137-3.175T8.1 2.788T12 2t3.9.788t3.175 2.137T21.213 8.1T22 12t-.788 3.9t-2.137 3.175t-3.175 2.138T12 22m0-2q3.325 0 5.663-2.337T20 12t-2.337-5.663T12 4T6.337 6.338T4 12t2.338 5.663T12 20"
            />
        </svg>
    );
}

export function MaterialSymbolsCalendarClock(
    props: JSX.IntrinsicElements["svg"],
) {
    return (
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" {...props}>
            <path
                fill="currentColor"
                d="M5 22q-.825 0-1.412-.587T3 20V6q0-.825.588-1.412T5 4h1V2h2v2h8V2h2v2h1q.825 0 1.413.588T21 6v4.675q0 .425-.288.713t-.712.287t-.712-.288t-.288-.712V10H5v10h5.8q.425 0 .713.288T11.8 21t-.288.713T10.8 22zm13 1q-2.075 0-3.537-1.463T13 18t1.463-3.537T18 13t3.538 1.463T23 18t-1.463 3.538T18 23m1.675-2.625l.7-.7L18.5 17.8V15h-1v3.2z"
            />
        </svg>
    );
}

export function MaterialSymbolsDoneAllRounded(
    props: JSX.IntrinsicElements["svg"],
) {
    return (
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width="1em"
            height="1em"
            viewBox="0 0 24 24"
            {...props}>
            <path
                fill="currentColor"
                d="M1.75 13.05q-.3-.3-.288-.7t.313-.7q.3-.275.7-.288t.7.288l3.55 3.55l.35.35l.35.35q.3.3.288.7t-.313.7q-.3.275-.7.288T6 17.3zm10.6 2.125l8.5-8.5q.3-.3.7-.287t.7.312q.275.3.288.7t-.288.7l-9.2 9.2q-.3.3-.7.3t-.7-.3L7.4 13.05q-.275-.275-.275-.687t.275-.713q.3-.3.713-.3t.712.3zm4.225-7.05L13.05 11.65q-.275.275-.687.275t-.713-.275q-.3-.3-.3-.712t.3-.713L15.175 6.7q.275-.275.688-.275t.712.275q.3.3.3.712t-.3.713"
            />
        </svg>
    );
}

export function InfoIcon(props: JSX.IntrinsicElements["svg"]) {
    return (
        <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            {...props}
            aria-label="small i inside circle">
            <path
                fill="currentColor"
                d="M11 17h2v-6h-2zm1-8q.425 0 .713-.288T13 8t-.288-.712T12 7t-.712.288T11 8t.288.713T12 9m0 13q-2.075 0-3.9-.788t-3.175-2.137T2.788 15.9T2 12t.788-3.9t2.137-3.175T8.1 2.788T12 2t3.9.788t3.175 2.137T21.213 8.1T22 12t-.788 3.9t-2.137 3.175t-3.175 2.138T12 22m0-2q3.35 0 5.675-2.325T20 12t-2.325-5.675T12 4T6.325 6.325T4 12t2.325 5.675T12 20m0-8"
            />
        </svg>
    );
}

export function Hamburger() {
    return (
        <div class="hamburger">
            <span></span>
            <span></span>
            <span></span>
        </div>
    );
}

export function TrendingUp(_props: JSX.IntrinsicElements["svg"]) {
    return <MaterialSymbolsArrowDownwardRounded class="rotate-180" />;
}

export function TrendingDown(_props: JSX.IntrinsicElements["svg"]) {
    return <MaterialSymbolsArrowDownwardRounded />;
}

export function SvgSpinnersPulseRing(props: JSX.IntrinsicElements["svg"]) {
    return (
        <>
            <span hidden>
                Icon from SVG Spinners by Utkarsh Verma -
                https://github.com/n3r4zzurr0/svg-spinners/blob/main/LICENSE
            </span>
            <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                {...props}>
                <path
                    fill="currentColor"
                    d="M12,1A11,11,0,1,0,23,12,11,11,0,0,0,12,1Zm0,20a9,9,0,1,1,9-9A9,9,0,0,1,12,21Z"
                    transform="matrix(0 0 0 0 12 12)">
                    <animateTransform
                        attributeName="transform"
                        calcMode="spline"
                        dur="1.2s"
                        keySplines=".52,.6,.25,.99"
                        repeatCount="indefinite"
                        type="translate"
                        values="12 12;0 0"
                    />
                    <animateTransform
                        additive="sum"
                        attributeName="transform"
                        calcMode="spline"
                        dur="1.2s"
                        keySplines=".52,.6,.25,.99"
                        repeatCount="indefinite"
                        type="scale"
                        values="0;1"
                    />
                    <animate
                        attributeName="opacity"
                        calcMode="spline"
                        dur="1.2s"
                        keySplines=".52,.6,.25,.99"
                        repeatCount="indefinite"
                        values="1;0"
                    />
                </path>
            </svg>
        </>
    );
}



export function SvgSpinnersPulse(props: JSX.IntrinsicElements['svg']) {
    return (
        <>
            <span hidden>
                Icon from SVG Spinners by Utkarsh Verma - https://github.com/n3r4zzurr0/svg-spinners/blob/main/LICENSE
            </span>
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" {...props}>
                <circle cx="12" cy="12" r="0" fill="currentColor">
                    <animate attributeName="r" calcMode="spline" dur="1.2s" keySplines=".52,.6,.25,.99" repeatCount="indefinite" values="0;11"/>
                    <animate attributeName="opacity" calcMode="spline" dur="1.2s" keySplines=".52,.6,.25,.99" repeatCount="indefinite" values="1;0"/>
                </circle>
            </svg>
        </>
    )
}


export function IcRoundCheck(props: JSX.IntrinsicElements['svg']) {
    return (
        <>
            <span hidden>
                Icon from Google Material Icons by Material Design Authors - https://github.com/material-icons/material-icons/blob/master/LICENSE
            </span>
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" {...props}>
                <path fill="currentColor" d="M9 16.17L5.53 12.7a.996.996 0 1 0-1.41 1.41l4.18 4.18c.39.39 1.02.39 1.41 0L20.29 7.71a.996.996 0 1 0-1.41-1.41z"/>
            </svg>
        </>
    )
}