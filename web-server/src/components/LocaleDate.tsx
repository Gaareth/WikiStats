import { createSignal, onMount } from "solid-js";

interface Props {
    date: Date;
    formatOption?: Intl.DateTimeFormatOptions;
}

export default function LocaleDate(props: Props) {
    let [date, setDate] = createSignal(props.date);
    let [formatOption, setFormatOption] = createSignal(props.formatOption);

    // Yes, formatter should be derived from formatOption() so it updates when formatOption changes.
    const formatter = () => new Intl.DateTimeFormat(undefined, formatOption());

    onMount(() => {
        // convert to locale time in the browser
        setDate(new Date(date().toUTCString()));
        setFormatOption({
            ...formatOption(),
            timeZone: undefined,
            timeZoneName: undefined,
        });
    });

    return (
        <time class="lg:text-nowrap" datetime={date().toISOString()}>
            {formatter().format(date())}
        </time>
    );
}
