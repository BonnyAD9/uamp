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
