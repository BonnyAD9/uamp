import { getSongsHeader } from "../pages.js";
import { displaySort, getHeaderlessTable } from "../tables.js";
import VirtualTable from "../virtual-table.js";
import Screen from "./screen.js";

export default class LibraryScreen extends Screen {
    constructor() {
        super("library-screen-template");
        this.table = null;
    }

    onReady() {
        this.table = new VirtualTable(
            () => app.library.songs.get(),
            "library-screen",
            ".songs tbody",
            () => app.player.getPlayingId(),
        );

        this.dom = {};
        this.#spawnElements();
    }

    /**
     * Sorts the library songs by the given key.
     * @param {string} key - key to sort by
     */
    sort(key) {
        const songs = app.library.sortLibrary(key);
        this.table.render();
        displaySort(this.dom.header, songs.key, songs.ascending);
    }

    #spawnElements() {
        const header = getSongsHeader((key) => this.sort(key));
        this.querySelector(".header").appendChild(header);
        this.dom.header = header;

        const table = getHeaderlessTable((e) => app.libraryClick(e));
        table.classList.add("with-song-context");
        this.querySelector(".screen-wrapper").appendChild(table);
    }
}
