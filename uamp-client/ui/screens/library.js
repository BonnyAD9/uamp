import { getSongsHeader } from "../pages.js";
import { getHeaderlessTable } from "../tables.js";
import VirtualTable from "../virtual-table.js";
import Screen from "./screen.js";

export default class LibraryScreen extends Screen {
    constructor() {
        super("library-screen-template");

        this.libraryTable = new VirtualTable(
            () => app.library.songs.get(),
            "library-screen",
            ".songs tbody",
            () => app.player.getPlayingId(),
        );
    }

    onReady() {
        this.dom = {};
        this.#spawnElements();
    }

    #spawnElements() {
        const header = getSongsHeader((key) => app.sortSongs(key));
        this.querySelector(".header").appendChild(header);

        const table = getHeaderlessTable((e) => app.libraryClick(e));
        table.classList.add("with-song-context");
        this.querySelector(".screen-wrapper").appendChild(table);
    }
}
