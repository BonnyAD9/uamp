import Sorter from "../../library/sorter.js";
import { genericDisplayAlbums, getCustomHeader } from "../pages.js";
import { displayAlbumsSort } from "../tables.js";
import Screen from "./screen.js";

export default class AlbumsScreen extends Screen {
    constructor() {
        super("albums-screen-template");
    }

    onReady() {
        this.dom = {
            list: this.querySelector(".list"),
        };
        this.#spawnElements();
    }

    /**
     * Displays the given albums on the albums page.
     * @param {Sorter} albums - albums to be displayed
     */
    display(albums) {
        genericDisplayAlbums(this.dom.list, albums.get());
        displayAlbumsSort(albums.key, albums.ascending);
    }

    #spawnElements() {
        const labels = ["Year", "Name", "Artist", "Songs"];
        const header = getCustomHeader(labels, (key) => app.sortAlbums(key));
        this.querySelector(".header").appendChild(header);
    }
}
