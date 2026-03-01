import {
    albumClickHandler,
    displayAlbums,
    displaySongs,
    songClickHandler,
} from "../pages.js";
import { displaySort, getTable } from "../tables.js";
import Screen from "./screen.js";

export default class ArtistScreen extends Screen {
    constructor() {
        super("artist-screen-template");
    }

    onReady() {
        this.dom = {
            name: this.querySelector(".info .name"),
            other: this.querySelector(".info .other"),
            albums: this.querySelector(".list"),
        };
        this.artist = null;
        this.#spawnTable();
        this.#setupListeners();
    }

    onNavigate(args) {
        if (!args?.id) return;

        const artist = app.library.allArtists[args.id];
        this.open(artist);
    }

    /**
     * Opens the given artist.
     * @param {Artist|null} artist - artist to be displayed on the page
     */
    open(artist) {
        if (!artist) return;

        this.artist = artist;
        app.artist = artist;
        this.dom.name.textContent = artist.name;
        this.dom.other.textContent = artist.getOtherDetails();

        this.#display();
    }

    /**
     * Sorts the artist songs by the given key.
     * @param {string} key - key to sort the songs by
     */
    sortSongs(key) {
        if (!this.artist) return;
        this.artist.songs.toggleSort(key);
        this.#display();
    }

    #spawnTable() {
        const table = getTable(
            (e) => songClickHandler(e, this.artist.songs.get()),
            (key) => this.sortSongs(key),
        );
        table.classList.add("with-song-context");

        this.querySelector(".screen-wrapper").appendChild(table);
        this.dom.songs = table;
    }

    #setupListeners() {
        this.dom.albums.addEventListener("click", albumClickHandler);
    }

    #display() {
        const id = app.player.getPlayingId();
        displaySongs(this.dom.songs, this.artist.songs.get(), true, id);
        displayAlbums(this.dom.albums, this.artist.albums);
        displaySort(
            this.dom.songs,
            this.artist.songs.key,
            this.artist.songs.ascending,
        );
    }
}
