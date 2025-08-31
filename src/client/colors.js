function rgbToHsl(r, g, b) {
    r /= 255;
    g /= 255;
    b /= 255;

    const max = Math.max(r, g, b), min = Math.min(r, g, b);
    let h, s, l = (max + min) / 2;

    if (max === min) {
        h = s = 0;
    } else {
        const d = max - min;
        s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
        switch (max) {
            case r:
                h = ((g - b) / d) + (g < b ? 6 : 0);
                break;
            case g:
                h = ((b - r) / d) + 2;
                break;
            case b:
                h = ((r - g) / d) + 4;
                break;
        }
        h /= 6;
    }
    return [h, s, l];
}

function hslToRgb(h, s, l) {
    let r, g, b;

    if (s === 0) {
        r = g = b = l;
    } else {
        const hue2rgb = (p, q, t) => {
            if (t < 0) t += 1;
            if (t > 1) t -= 1;
            if (t < 1 / 6) return p + (q - p) * 6 * t;
            if (t < 1 / 2) return q;
            if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
            return p;
        };

        const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
        const p = 2 * l - q;

        r = hue2rgb(p, q, h + 1 / 3);
        g = hue2rgb(p, q, h);
        b = hue2rgb(p, q, h - 1 / 3);
    }

    return [Math.round(r * 255), Math.round(g * 255), Math.round(b * 255)];
}

function rgbToCss(rgb) {
    return `rgb(${rgb[0]}, ${rgb[1]}, ${rgb[2]})`;
}

function rgbToRgbaCss(rgb, alpha = 1) {
    return `rgba(${rgb[0]}, ${rgb[1]}, ${rgb[2]}, ${alpha})`;
}

const colorThief = new ColorThief();

function setupDynamicColors(dynamic) {
    if (dynamic) {
        setDynamicColors();
        songIcon.onload = setDynamicColors;
    } else {
        resetColors()
        songIcon.onload = null;
    }
}

function applyThemeColor(color) {
    if ((getCookie('dynamicColor') ?? 'true') !== 'true')
        document.documentElement.style.setProperty('--primary', color);
}

function setDynamicColors() {
    const dominantRgb = colorThief.getColor(songIcon);
    const [h, s, _] = rgbToHsl(...dominantRgb);
    const root = document.documentElement;

    const bg = hslToRgb(h, s * 0.3, 0.1);
    root.style.setProperty('--bg', rgbToCss(bg))
    root.style.setProperty('--bg-trans', rgbToRgbaCss(bg, 0.6));

    const bgLight = hslToRgb(h, s * 0.4, 0.14);
    root.style.setProperty('--bg-light', rgbToCss(bgLight));
    root.style.setProperty('--bg-light-trans', rgbToRgbaCss(bgLight, 0.6));

    const bgLighter = hslToRgb(h, s * 0.5, 0.18);
    root.style.setProperty('--bg-lighter', rgbToCss(bgLighter));

    const bgDark = hslToRgb(h, s * 0.3, 0.07);
    root.style.setProperty('--bg-dark', rgbToCss(bgDark));
    root.style.setProperty('--bg-dark-trans', rgbToRgbaCss(bgDark, 0.6));

    root.style.setProperty('--primary', rgbToCss(hslToRgb(h, s, 0.5)));

    root.style.setProperty('--fg', rgbToCss(hslToRgb(h, s, 0.93)));
    root.style.setProperty('--fg2', rgbToCss(hslToRgb(h, s, 0.8)));
    root.style.setProperty('--fg-sec', rgbToCss(hslToRgb(h, s * 0.2, 0.53)));
    root.style.setProperty('--fg-sec2', rgbToCss(hslToRgb(h, s * 0.2, 0.4)));
}

function resetColors() {
    const root = document.documentElement;
    root.style.removeProperty('--bg');
    root.style.removeProperty('--bg-trans');
    root.style.removeProperty('--bg-lighter');
    root.style.removeProperty('--bg-light');
    root.style.removeProperty('--bg-light-trans');
    root.style.removeProperty('--bg-dark');
    root.style.removeProperty('--bg-dark-trans');
    root.style.removeProperty('--fg');
    root.style.removeProperty('--fg2');
    root.style.removeProperty('--fg-sec');
    root.style.removeProperty('--fg-sec2');

    const primary = getCookie('themeColor') ?? '#3acbaf';
    root.style.setProperty('--primary', primary);
}

if ((getCookie('dynamicColor') ?? 'true') === 'true') {
    songIcon.onload = setDynamicColors;
} else {
    const color = getCookie('themeColor') ?? '#3acbaf';
    document.documentElement.style.setProperty('--primary', color);
    songIcon.onload = null;
}
