import Sorter from "../../library/sorter.js";
import { genericDisplayAlbums, getCustomHeader } from "../pages.js";
import { displaySort } from "../tables.js";
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
        displaySort(this.dom.header, albums.key, albums.ascending);
    }

    /**
     * Sorts the albums by the given key.
     * @param {string} key - key to sort the albums by
     */
    sort(key) {
        const albums = app.library.sortAlbums(key);
        this.display(albums);
    }

    #spawnElements() {
        const labels = ["Year", "Name", "Artist", "Songs"];
        const header = getCustomHeader(labels, (key) => this.sort(key));
        this.querySelector(".header").appendChild(header);
        this.dom.header = header;
    }
}
