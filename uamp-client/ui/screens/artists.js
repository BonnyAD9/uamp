import Sorter from "../../library/sorter.js";
import { getCustomHeader } from "../pages.js";
import { displayArtistsSort } from "../tables.js";
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
        displayArtistsSort(artists.key, artists.ascending);
    }

    #spawnElements() {
        const labels = ["Name", "Albums", "Songs"];
        const header = getCustomHeader(labels, (key) => app.sortArtists(key));
        this.querySelector(".header").appendChild(header);
    }
}
