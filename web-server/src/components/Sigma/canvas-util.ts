import type { PlainObject } from "sigma/types";
import { big_num_format } from "../../utils";

// See: https://github.com/jacomyal/sigma.js/blob/main/packages/demo/src/canvas-utils.ts

/**
 * This function draw in the input canvas 2D context a rectangle.
 * It only deals with tracing the path, and does not fill or stroke.
 */
export function drawRoundRect(
    ctx: CanvasRenderingContext2D,
    x: number,
    y: number,
    width: number,
    height: number,
    radius: number,
): void {
    ctx.beginPath();
    ctx.moveTo(x + radius, y);
    ctx.lineTo(x + width - radius, y);
    ctx.quadraticCurveTo(x + width, y, x + width, y + radius);
    ctx.lineTo(x + width, y + height - radius);
    ctx.quadraticCurveTo(x + width, y + height, x + width - radius, y + height);
    ctx.lineTo(x + radius, y + height);
    ctx.quadraticCurveTo(x, y + height, x, y + height - radius);
    ctx.lineTo(x, y + radius);
    ctx.quadraticCurveTo(x, y, x + radius, y);
    ctx.closePath();
}

function drawDisc(context: CanvasRenderingContext2D, data: PlainObject) {
    const PADDING = 2;

    context.fillStyle = "#FFF";
    context.shadowOffsetX = 0;
    context.shadowOffsetY = 0;
    context.shadowBlur = 8;
    context.shadowColor = "#000";

    context.beginPath();
    context.arc(data.x, data.y, data.size + PADDING, 0, Math.PI * 2);
    context.closePath();
    context.fill();
}

/**
 * Custom hover renderer
 */
export function drawHover(
    context: CanvasRenderingContext2D,
    data: PlainObject,
    settings: PlainObject,
) {
    const TEXT_X_OFFSET = 10;

    const size = settings.labelSize;
    const font = settings.labelFont;
    const weight = settings.labelWeight;
    const subLabelSize = size - 1;

    const label = data.label;
    const num_links_text = "Links: " + big_num_format(data.num_links);
    const times_linked_text =
        "Times linked: " + big_num_format(data.times_linked);

    // Then we draw the label background
    context.beginPath();
    context.fillStyle = "#fff";
    context.shadowOffsetX = 0;
    context.shadowOffsetY = 2;
    context.shadowBlur = 8;
    context.shadowColor = "#000";

    context.font = `${weight} ${size}px ${font}`;
    const labelWidth = context.measureText(label).width;
    context.font = `${weight} ${subLabelSize}px ${font}`;
    const times_linkedWidth =
        data.times_linked !== undefined
            ? context.measureText(times_linked_text).width
            : 0;
    context.font = `${weight} ${subLabelSize}px ${font}`;
    const num_linksWidth =
        data.num_links !== undefined
            ? context.measureText(num_links_text).width
            : 0;

    const textWidth = Math.max(labelWidth, times_linkedWidth, num_linksWidth);

    const x = Math.round(data.x);
    const y = Math.round(data.y);
    const w = Math.round(textWidth + size / 2 + data.size + TEXT_X_OFFSET);
    let h = size + 4 + 4 + 4;
    if (data.num_links !== undefined) {
        h += subLabelSize + 4;
    }

    if (data.times_linked !== undefined) {
        h += subLabelSize;
    }

    drawRoundRect(context, x, y - size, w, h, 5);
    context.closePath();
    context.fill();
    context.shadowOffsetX = 0;
    context.shadowOffsetY = 0;
    context.shadowBlur = 0;

    // And finally we draw the labels
    context.fillStyle = "#000000";
    context.font = `${weight} ${size}px ${font}`;
    context.fillText(
        label,
        data.x + data.size + TEXT_X_OFFSET,
        data.y + size / 3,
    );

    // And finally we draw the labels
    // context.fillStyle = TEXT_COLOR;
    context.font = `${weight} ${subLabelSize}px ${font}`;

    if (data.num_links !== undefined) {
        context.fillText(
            num_links_text,
            data.x + data.size + TEXT_X_OFFSET,
            data.y + size / 3 + 4 + size,
        );
    }

    if (data.times_linked !== undefined) {
        context.fillText(
            times_linked_text,
            data.x + data.size + TEXT_X_OFFSET,
            data.y + size / 3 + 4 + size * 2,
        );
    }

    drawDisc(context, data);
}
