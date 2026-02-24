import { getSongsHeader } from "../pages.js";
import { getHeaderlessTable } from "../tables.js";
import VirtualTable from "../virtual-table.js";
import Screen from "./screen.js";

export default class PlaylistScreen extends Screen {
    constructor() {
        super("playlist-screen-template");

        this.table = null;
    }

    onReady() {
        this.table = new VirtualTable(
            () => app.player.getPlaylist(app.playlistTab).songs,
            "#playlist",
            ".playlist-stack.active tbody",
            () => app.player.getPlaylist(app.playlistTab).current,
        );
        this.table.playlist().autoScrolling();

        this.dom = {
            playlists: this.querySelector(".playlist-wrapper"),
            tabs: this.querySelector(".tabs .tabs-wrapper"),
        };
        this.#spawnElements();
    }

    /**
     * Displays the playlist stack.
     * @param {number} n - number of playlists
     */
    displayStack(n) {
        this.dom.tabs
            .querySelectorAll(".tab:not(#playingPlaylist)")
            .forEach((tab) => tab.remove());
        this.dom.playlists
            .querySelectorAll(".playlist-stack")
            .forEach((table, i) => i !== 0 && table.remove());

        for (let i = 1; i <= n; i++) {
            this.push();
        }
    }

    /**
     * Shows the playlist with the given ID.
     * @param {number} id - ID of the playlist to show.
     */
    show(id) {
        const tabs = this.dom.tabs.querySelectorAll(".tab");
        const stacks = this.dom.playlists.querySelectorAll(".playlist-stack");
        for (let i = 0; i < tabs.length; i++) {
            tabs[i].classList.toggle("active", i === id);
            stacks[i].classList.toggle("active", i === id);
        }
    }

    /** Pushes new empty table and tab button to the playlist screen. */
    push() {
        const table = this.#getTable();

        const playing = this.dom.playlists.querySelector(".playlist-stack");
        this.dom.playlists.insertBefore(table, playing);

        const tab = document.createElement("button");
        tab.classList.add("tab");

        const id = this.dom.tabs.querySelectorAll(".tab").length;
        tab.textContent = `-${id}`;
        tab.onclick = () => app.setPlaylistTab(id);
        this.dom.tabs.appendChild(tab);
    }

    /** Pops playlist table and tab button form the playlist screen. */
    pop() {
        this.remove(0);
    }

    /**
     * Reorders playlists based on the igven indices.
     * @param {number[]} indices - reorder indices containing all playlists
     */
    reorder(indices) {
        const stacks = this.dom.playlists.querySelectorAll(".playlist-stack");
        const tables = Array.from(stacks);

        const reordered = indices.map((i) => tables[i]);
        reordered.forEach((table) => this.dom.playlists.appendChild(table));
    }

    /**
     * Removes given playlist and tab from the playlist screen.
     * @param {number} id - ID of the playlist to be removed
     */
    remove(id = 0) {
        const tables = this.dom.playlists.querySelectorAll(".playlist-stack");
        if (tables.length < 2 && tables.length <= id) return;
        tables[id].remove();

        const tabs = this.dom.tabs.querySelectorAll(
            ".tab:not(#playingPlaylist)",
        );
        if (tabs.length == 0) return;
        tabs[tabs.length - 1].remove();
    }

    /**
     * Removes given row from the playing playlist.
     * @param {number} id - ID of the table row
     */
    removeRow(id) {
        const table = this.dom.playlists.querySelector(".playlist-stack tbody");
        const rows = table.querySelectorAll("tr");
        if (rows.length <= id) return;

        const row = rows[id];
        if (row.classList.contains("active") && row.nextSibling !== null) {
            row.nextSibling.classList.add("active");
        }
        row.remove();
    }

    #spawnElements() {
        const header = getSongsHeader(null);
        this.querySelector(".header").appendChild(header);

        const table = this.#getTable();
        this.querySelector(".playlist-wrapper").appendChild(table);
    }

    #getTable() {
        const table = getHeaderlessTable(
            (e) => app.playlistClick(e),
            ["playlist-stack", "active"],
        );
        table.classList.add("with-playlist-context");
        return table;
    }
}
