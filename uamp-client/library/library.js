import Album from "./album.js";
import Artist from "./artist.js";
import Song from "./song.js";
import Sorter from "./sorter.js";

/**
 * @typedef {Object} LibraryData
 * @property {object[]} songs
 * @property {object[]} tmp_songs
 * @property {object[]} artists
 * @property {object[]} albums
 */

export default class Library {
    /**
     * Creates new library from the given library data
     * @param {LibraryData} library
     */
    constructor({ songs, tmp_songs, artists, albums }) {
        /** @type {Song[]} */
        this.allSongs = songs.map((s, i) => Song.from(i, s));
        /** @type {Object<string, Album>} */
        this.allAlbums = Object.entries(albums).reduce((acc, [id, album]) => {
            acc[id] = Album.from(album, this.allSongs);
            return acc;
        }, {});
        /** @type {Object<string, Artist>} */
        this.allArtists = Object.entries(artists).reduce((acc, [id, a]) => {
            acc[id] = Artist.from(a, this.allAlbums, this.allSongs);
            return acc;
        }, {});

        /** @type {Song[]} */
        this.tmpSongs = tmp_songs.map((s, i) => Song.from(-i - 1, s));

        /** @type {Sorter} */
        this.songs = new Sorter("id");
        /** @type {Sorter} */
        this.albums = new Sorter("year", [], false, ["name", "artist"]);
        /** @type {Sorter} */
        this.artists = new Sorter("name");

        this.#generate();
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
     * @returns {Album[]} all library albums
     */
    getAlbums() {
        return Object.values(this.allAlbums);
    }

    /**
     * @returns {Artist[]} all library artists
     */
    getArtists() {
        return Object.values(this.allArtists);
    }

    /**
     * Gets artist by its name
     * @param {string} name - artist name
     * @returns {Artist|null} found artist or null
     */
    getArtistByName(name) {
        return this.getArtists().find(
            (a) => a.name.toLowerCase() === name.toLowerCase(),
        );
    }

    /**
     * Gets album by its keys
     * @param {string} artist - name of the author of the album
     * @param {string} name - album name
     * @returns {Album|null} found album or null
     */
    getAlbumByKey(artist, name) {
        return this.getAlbums().find(
            (a) =>
                a.name.toLowerCase() === name.toLowerCase() &&
                a.artist.toLowerCase() === artist.toLowerCase(),
        );
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
                s.title?.toLowerCase()?.includes(q) ||
                s.album_artist?.toLowerCase()?.includes(q) ||
                s.artists.some((a) => a.toLowerCase().includes(q)) ||
                s.album?.toLowerCase()?.includes(q)
            );
        });
        this.songs.set(queried);
    }

    searchAlbums(query) {
        const q = query.trim().toLowerCase();
        if (!q) {
            this.albums.set(this.getAlbums());
            return;
        }

        const filtered = this.getAlbums().filter((a) => {
            return (
                a.artist.toLowerCase().includes(q) ||
                a.name.toLowerCase().includes(q)
            );
        });
        this.albums.set(filtered);
    }

    searchArtists(query) {
        const q = query.trim().toLowerCase();
        if (!q) {
            this.artists.set(this.getArtists());
            return;
        }

        const filtered = this.getArtists().filter((a) =>
            a.name.toLowerCase().includes(q),
        );
        this.artists.set(filtered);
    }

    /** Generates albums and artists */
    #generate() {
        for (const song of this.allSongs) {
            if (song.deleted) continue;
            this.songs.push(song);
        }

        this.getAlbums().forEach((album) => album.sortByTrack());
        this.albums.set(this.getAlbums());

        this.getArtists().forEach((artist) => artist.sortAlbums());
        this.artists.set(this.getArtists());
    }
}
