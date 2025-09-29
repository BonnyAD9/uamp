import Album from "./album.js";
import Artist from "./artist.js";
import Song from "./song.js";
import Songs from "./songs.js";

export default class Library {
    /**
     * Creates new library from the given library data
     * @param {{ songs: object[], tmp_songs: object[] }} library
     */
    constructor(library) {
        /** @type {Song[]} */
        this.allSongs = library.songs.map((s, i) => Song.from(i, s));
        /** @type {Song[]} */
        this.tmpSongs = library.tmp_songs.map((s, i) => Song.from(-i - 1, s));

        /** @type {Songs} */
        this.songs = new Songs();
        /** @type {Album[]} */
        this.albums = [];
        /** @type {Artist[]} */
        this.artists = [];

        this.#generate();

        const input = document.getElementById("library-search");
        if (input.value) {
            this.searchLibrary(input.value);
        }
    }

    /**
     * Gets song from the library based on the given id
     * @param {number} id - id of the song
     * @returns {?Song} song when found, else null
     */
    getSong(id) {
        if (id < 0) return this.tmpSongs[-id - 1];
        return this.allSongs[id];
    }

    /**
     * Applies the library search query and saves the result to songs.
     * Resets the search when the query is empty.
     * @param {string} query - search query
     */
    searchLibrary(query) {
        const q = query.trim().toLowerCase();
        if (!q) {
            this.songs.set(this.allSongs.filter((s) => !s.deleted));
            return;
        }

        const queried = this.allSongs.filter((s) => {
            if (s.deleted) return false;

            return (
                s.title.toLowerCase().includes(q) ||
                s.artist.toLowerCase().includes(q) ||
                s.album.toLowerCase().includes(q)
            );
        });
        this.songs.set(queried);
    }

    /** Generates albums and artists (TODO: try refactor + album push()) */
    #generate() {
        const albums = new Map();
        const artists = new Map();

        for (const song of this.allSongs) {
            if (song.deleted) continue;

            this.songs.push(song);
            const artistKey = song.artist.trim().toLowerCase();
            if (!artists.has(artistKey)) {
                artists.set(artistKey, new Artist(song.artist));
            }

            const artist = artists.get(artistKey);
            artist.songs.push(song);

            const albumKey = `${song.album.trim().toLowerCase()}::${artistKey}`;
            if (!albums.has(albumKey)) {
                albums.set(
                    albumKey,
                    new Album(song.album, song.artist, song.year),
                );
                artist.albums.push(albums.get(albumKey));
            }
            albums.get(albumKey).songs.push(song);
        }

        this.albums = Array.from(albums.values());
        this.albums.forEach((album) => album.sortByTrack());

        this.artists = Array.from(artists.values());
        this.artists.sort((a, b) => a.name.localeCompare(b.name));
        this.artists.forEach((artist) => artist.sortAlbums());
    }
}
