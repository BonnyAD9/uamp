import Duration from "../../helper/duration.js";

export default class Menu extends HTMLElement {
    constructor() {
        super();
    }

    connectedCallback() {
        const nodes = Array.from(this.childNodes);
        this.innerHTML = `
            <div class="logo">
                <img src="assets/svg/icon.svg" alt="uamp logo">
                <h1>Uamp</h1>
            </div>
            <nav><span id="menuLabel"></span></nav>
        `;

        this.dom = {
            label: this.querySelector("#menuLabel"),
        };

        const nav = this.querySelector("nav");
        nodes.forEach((node) => nav.insertBefore(node, this.dom.label));
        this.dom.navs = this.querySelectorAll("[data-screen]");

        this.#setupListeners();
    }

    /**
     * Highlights nav item with the given screen.
     * @param {string} screen - screen to highlight item for
     */
    highlight(screen) {
        this.dom.navs.forEach((p) =>
            p.classList.toggle("active", p.dataset.screen === screen),
        );
    }

    /**
     * Displays songs stats to the label - songs count and total length
     * @param {Song[]} songs - songs list to display stats to the label
     */
    setLabel(songs) {
        if (songs.length === 0) {
            this.dom.label.textContent = "";
            return;
        }

        const len = this.#songsTotalLen(songs).format();
        this.dom.label.textContent = `${songs.length} songs • ${len}`;
    }

    #setupListeners() {
        this.addEventListener("click", (e) => {
            const item = e.target.closest("[data-screen]");
            if (!item) return;

            this.querySelectorAll("[data-screen]").forEach((p) =>
                p.classList.remove("active"),
            );
            item.classList.add("active");

            app.navigateTo(item.dataset.screen);
        });
    }

    /**
     * Gets songs total duration
     * @param {Song[]} songs - songs to get the total for
     * @returns {Duration} - total duration
     */
    #songsTotalLen(songs) {
        return songs.reduce(
            (acc, song) => acc.add(song.length),
            new Duration(0, 0),
        );
    }
}
