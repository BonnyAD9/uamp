import { displaySongs, genericDisplayAlbums } from "../pages.js";
import { getTable } from "../tables.js";
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
        this.#spawnTable();
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

        app.artist = artist;
        this.dom.name.textContent = artist.name;
        this.dom.other.textContent = artist.getOtherDetails();

        const id = app.player.getPlayingId();
        displaySongs(this.dom.songs, artist.songs.get(), true, id);
        genericDisplayAlbums(this.dom.albums, artist.albums);
    }

    #spawnTable() {
        const table = getTable(
            (e) => app.artistSongClick(e),
            (key) => app.sortArtistSongs(key),
        );
        table.classList.add("with-song-context");

        this.querySelector(".screen-wrapper").appendChild(table);
        this.dom.songs = table;
    }
}
