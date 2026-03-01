import Api from "./api.js";
import { contextMenu } from "./ui/context-menu.js";

/**
 * Default hotkeys map
 */
export const defaultHotkeys = {
    Space: "pp",
    K: "pp",
    J: "ps",
    L: "ns",
    M: "mute",
    ArrowUp: "vu",
    ArrowDown: "vd",
    ArrowRight: "ff",
    ArrowLeft: "rw",
};

export default class HotkeyManager {
    /**
     * Creates new hotkey manager with the given configuration.
     * @param {Object} config - map of hotkeys and ctrl API actions
     */
    constructor(config) {
        this.config = config;

        document.addEventListener("keydown", (e) => this.#handleKeydown(e));
    }

    /**
     * Update the hotkey configuration.
     * @param {Object} config - new map of hotkeys and ctrl API actions
     */
    setConfig(config) {
        this.config = config;
    }

    #handleKeydown(e) {
        if (e.key === "Escape") {
            e.preventDefault();
            this.#escHandle(e);
            return;
        }

        if (e.target.tagName !== "BODY") return;

        const key = this.#getKeyStr(e);
        const action = this.config[key];
        if (action) {
            e.preventDefault();
            Api.ctrl(action);
        }
    }

    #getKeyStr(e) {
        if (["Control", "Alt", "Meta", "Shift"].includes(e.key)) return null;

        const keys = [];
        if (e.ctrlKey) keys.push("Ctrl");
        if (e.altKey) keys.push("Alt");
        if (e.shiftKey) keys.push("Shift");
        if (e.metaKey) keys.push("Meta");

        let key = e.key;
        if (key === " ") {
            key = "Space";
        } else if (key.length === 1) {
            key = key.toUpperCase();
        }

        keys.push(key);
        return keys.join("+");
    }

    #escHandle(e) {
        e.target.blur();
        contextMenu.hide();
        document.querySelector("player-bar.bar").classList.remove("active");
    }
}
