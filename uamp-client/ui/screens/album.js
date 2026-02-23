import Album from "../../library/album.js";
import { displaySongs } from "../pages.js";
import { getTable } from "../tables.js";
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
        this.#spawnTable();
    }

    onNavigate(args) {
        if (!args?.id) return;

        const album = app.library.allAlbums[args.id];
        if (!album) return;

        app.album = album;
        this.dom.cover.src = Album.getCover(album.artist, album.name);
        this.dom.title.textContent = album.name;
        this.dom.artist.textContent = album.artist;

        const other = album.year !== null ? `${album.getYear()}  •  ` : "";
        this.dom.other.textContent = `${other}${album.songs.len()} songs`;

        const id = app.player.getPlayingId();
        displaySongs(this.dom.songs, album.songs.get(), false, id);
    }

    #spawnTable() {
        const table = getTable(
            (e) => app.albumSongClick(e),
            (key) => app.sortAlbumSongs(key),
        );
        table.classList.add("with-song-context");

        const col = table.querySelector(".col-img");
        col.className = "col-empty";

        this.querySelector(".album-detail-wrapper").appendChild(table);
        this.dom.songs = table;
    }
}
