import type Graph from "graphology";
import {
    createContext,
    createSignal,
    useContext,
    type Accessor,
} from "solid-js";
import {
    DEFAULT_RENDER_SETTINGS,
    type GraphRenderSettings,
} from "./SigmaGraph";

const SigmaContext = createContext<ReturnType<typeof makeSigmaContext>>();

interface SigmaContextProps {
    graph: Graph;
    loaded?: Accessor<boolean>;
    renderSettings?: GraphRenderSettings;
    children: any;
}

function makeSigmaContext(props: SigmaContextProps) {
    const [getGraph, setGraph] = createSignal(props.graph);

    const [renderSettings, setRenderSettings] =
        createSignal<GraphRenderSettings>(
            props.renderSettings ?? DEFAULT_RENDER_SETTINGS,
        );

    let [loaded] = createSignal(true);
    if (props.loaded !== undefined) {
        loaded = props.loaded;
    }

    return {
        graph: { getGraph, setGraph, loaded },
        renderSettings: { renderSettings, setRenderSettings },
    };
}

export function SigmaProvider(props: SigmaContextProps) {
    return (
        <SigmaContext.Provider value={makeSigmaContext(props)}>
            {props.children}
        </SigmaContext.Provider>
    );
}

export function useSigma() {
    return useContext(SigmaContext);
}
