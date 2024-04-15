declare module "gauntlet:renderer" {
    import { ReactNode } from "react";

    const render: (entrypointId: string, renderLocation: RenderLocation, component: ReactNode) => UiWidget;
    export { render };
}