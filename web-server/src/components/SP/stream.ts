import type { Setter } from "solid-js";

interface FetchProps {
    start: string;
    end: string;
    wiki_name: string;
    setStreamData: Setter<streamData>;
}

export type streamData = {
    visited: number;
    elapsed_ms: number;
    paths?: string[][];
};

export const fetchShortestPathClient = async (
    props: FetchProps,
): Promise<streamData> => {
    if (import.meta.env.SSR) {
        return {
            visited: 0,
            elapsed_ms: 0,
        };
    }

    const url = `/api/${props.wiki_name}/sp?start=${props.start}&end=${props.end}&stream=true`;

    const resp = await fetch(url);
    console.log(resp);

    if (!resp.ok) {
        let error_msg = await resp.text();
        if (error_msg.length == 0) {
            error_msg = resp.statusText;
        }
        throw Error(error_msg);
    }

    if (resp.body == null) {
        throw Error("Response body is null");
    }

    // throw Error(resp.body);

    //   for await (const chunk of resp.body) {}

    const reader = resp.body.getReader();
    let json_line;

    await parse_stream(reader, (text_line) => {
        json_line = JSON.parse(text_line);
        props.setStreamData(json_line);
    });

    if (json_line === undefined) {
        throw Error("Response stream was empty");
    }

    return json_line;
};

export async function parse_stream(
    reader: ReadableStreamDefaultReader<Uint8Array>,
    callback: (line: string) => any,
) {
    let done;
    let buffered_text_line = "";

    while (!done) {
        //  ({ value, done } = await reader.read()); what's that syntax??
        const { value, done } = await reader.read();
        if (done) {
            break;
        }

        const json_chunk = new TextDecoder().decode(value);
        console.log(json_chunk);

        for (let text_line of json_chunk.split("\n")) {
            try {
                if (buffered_text_line.length > 0) {
                    text_line = buffered_text_line + text_line;
                }
                await callback(text_line);
                // setVisited(json_line["visited"]);
                // setVisited(json_line["elapsed_ms"]);
                // console.log(json_line);

                // setPerSec(json_line["per_sec"]);
            } catch {
                console.log("Failed parsing json: " + text_line);
                if (!text_line.includes("\n")) {
                    buffered_text_line = text_line;
                }
            }
        }
    }
}
