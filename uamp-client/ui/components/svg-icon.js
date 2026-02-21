class SvgIcon extends HTMLElement {
    constructor() {
        super();

        this._readyPromise = new Promise((resolve) => {
            this._resolveReady = resolve;
        });
    }

    async connectedCallback() {
        const src = this.getAttribute("src");
        if (!src) {
            this._resolveReady();
            return;
        }

        const svg = await fetch(`/app/${src}`).then((res) => {
            if (!res.ok) return null;
            return res.text();
        });
        if (svg) this.innerHTML = svg;

        this._resolveReady();
    }

    /**
     * Checks if the svg icon finished loading.
     * @returns {Promise}
     */
    waitReady() {
        return this._readyPromise;
    }

    /**
     * Triggers SMIL animation with given ID.
     * @param {string} id - the ID of the animate tag
     */
    triggerAnimation(id) {
        const anim = this.querySelector(`#${id}`);
        if (anim && typeof anim.beginElement === "function") {
            anim.beginElement();
        }
    }
}

if (!customElements.get("svg-icon")) {
    customElements.define("svg-icon", SvgIcon);
}
