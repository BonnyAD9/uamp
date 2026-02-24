import Album from "../../library/album.js";
import { displaySongs } from "../pages.js";
import { displaySort, getTable } from "../tables.js";
import Screen from "./screen.js";

export default class AlbumScreen extends Screen {
    constructor() {
        super("album-screen-template");
    }

    onReady() {
        this.dom = {
            cover: this.querySelector(".info img"),
            title: this.querySelector(".info .name"),
            artist: this.querySelector(".info .artist"),
            other: this.querySelector(".info .other"),
        };
        this.album = null;
        this.#spawnTable();
    }

    /**
     * Handles navigation to album page
     * @param {Object} args - page arguments
     */
    onNavigate(args) {
        if (!args?.id) return;

        const album = app.library.allAlbums[args.id];
        this.open(album);
    }

    /**
     * Opens the given album.
     * @param {Album|null} album - album to be displayed on the page
     */
    open(album) {
        if (!album) return;

        this.album = album;
        app.album = album;
        this.dom.cover.src = Album.getCover(album.artist, album.name);
        this.dom.title.textContent = album.name;
        this.dom.artist.textContent = album.artist;

        const other = album.year !== null ? `${album.getYear()}  •  ` : "";
        this.dom.other.textContent = `${other}${album.songs.len()} songs`;

        this.#display();
    }

    /**
     * Sorts the album songs by the given key.
     * @param {string} key - key to sort the songs by
     */
    sort(key) {
        if (!this.album) return;
        this.album.songs.toggleSort(key);
        this.#display();
    }

    #spawnTable() {
        const table = getTable(
            (e) => app.albumSongClick(e),
            (key) => this.sort(key),
        );
        table.classList.add("with-song-context");

        const col = table.querySelector(".col-img");
        col.className = "col-empty";

        this.querySelector(".album-detail-wrapper").appendChild(table);
        this.dom.songs = table;
    }

    #display() {
        const id = app.player.getPlayingId();
        displaySongs(this.dom.songs, this.album.songs.get(), false, id);
        displaySort(
            this.dom.songs,
            this.album.songs.key,
            this.album.songs.ascending,
        );
    }
}
