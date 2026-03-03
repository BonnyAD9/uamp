import Sorter from "../../library/sorter.js";
import { getCustomHeader } from "../pages.js";
import { displaySort } from "../tables.js";
import Screen from "./screen.js";

export default class ArtistsScreen extends Screen {
    constructor() {
        super("artists-screen-template");
    }

    onReady() {
        this.dom = {
            list: this.querySelector(".songs tbody"),
        };
        this.#spawnElements();
        this.#setupListeners();
    }

    /**
     * Displays the given artists on the artists page.
     * @param {Sorter} artists - artists to be displayed
     */
    display(artists) {
        this.dom.list.innerHTML = "";
        artists.get().forEach((artist) => {
            const row = artist.getTableRow();
            row.dataset.index = artist.id;
            this.dom.list.appendChild(row);
        });
        displaySort(this.dom.header, artists.key, artists.ascending);
    }

    /**
     * Sorts the artists by the given key.
     * @param {string} key - key to sort the artists by
     */
    sort(key) {
        const artists = app.library.sortArtists(key);
        this.display(artists);
    }

    #spawnElements() {
        const labels = ["Name", "Albums", "Songs"];
        const header = getCustomHeader(labels, (key) => this.sort(key));
        this.querySelector(".header").appendChild(header);
        this.dom.header = header;
    }

    #setupListeners() {
        this.dom.list.addEventListener("click", (e) => this.#clickHandler(e));
    }

    #clickHandler(e) {
        const row = e.target.closest("tr");
        if (!row) return;

        const album = e.target.closest(".albums-preview .card");
        if (album) {
            app.navigateTo("album-detail", { id: album.dataset.index });
            return;
        }

        app.navigateTo("artist-detail", { id: row.dataset.index });
    }
}
