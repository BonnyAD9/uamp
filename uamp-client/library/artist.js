import Album from "./album.js";
import Song from "./song.js";
import Sorter from "./sorter.js";

const artistTemplate = document.getElementById("artist-template");

export default class Artist {
    /**
     * Creates new artist
     * @param {string} name
     * @param {Song[]} songs
     * @param {Album[]} albums
     */
    constructor(name, songs = [], albums = []) {
        /** @type {string} */
        this.name = name;
        /** @type {Sorter} */
        this.songs = new Sorter("album", songs, true, ["track"]);
        /** @type {Album[]} */
        this.albums = albums;
    }

    static from(obj, allAlbums, allSongs) {
        const albums = obj.albums.map((a, _) => allAlbums[a]);
        const singles = obj.singles.map((s, _) => allSongs[s]);
        const songs = [...albums.flatMap((a) => a.songs.get()), ...singles];
        return new Artist(obj.name, songs, albums);
    }

    /**
     * Gets albums and songs count string
     * @returns {string} the details string
     */
    getOtherDetails() {
        return `${this.albums.length} albums  •  ${this.songs.len()} songs`;
    }

    /**
     * Generates a table row with artist details
     * @returns {HTMLTableRowElement} - generated artist table row
     */
    getTableRow() {
        const cloned = artistTemplate.content.cloneNode(true);
        const row = cloned.querySelector("tr");

        row.querySelector(".artist").textContent = this.name;
        row.querySelector(".other").textContent = this.getOtherDetails();

        const albums = row.querySelector(".albums-preview");
        this.albums.forEach((album) => {
            const img = document.createElement("img");
            img.src = Album.getCover(album.artist, album.name, 64);
            img.title = album.name;
            img.dataset.index = album.id;
            albums.appendChild(img);
        });

        return row;
    }

    /**
     * Gets uamp query for filtering the artist
     * @returns {string} uamp query string
     */
    getQuery() {
        const s = (text) => text.replaceAll("/", "//");
        return `p=/${s(this.name)}/`;
    }

    /**
     * Sorts artists albums by release year
     */
    sortAlbums() {
        this.albums.sort((a, b) => b.year - a.year);
    }
}
