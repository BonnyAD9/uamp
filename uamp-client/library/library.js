import Album from "./album.js";
import Artist from "./artist.js";
import Song from "./song.js";
import Sorter from "./sorter.js";

export default class Library {
    /**
     * Creates new library from the given library data
     * @param {{ songs: object[], tmp_songs: object[] }} library
     */
    constructor(library) {
        /** @type {Song[]} */
        this.allSongs = library.songs.map((s, i) => Song.from(i, s));
        /** @type {Album[]} */
        this.allAlbums = [];
        /** @type {Artist[]} */
        this.allArtists = [];

        /** @type {Song[]} */
        this.tmpSongs = library.tmp_songs.map((s, i) => Song.from(-i - 1, s));

        /** @type {Sorter} */
        this.songs = new Sorter("id");
        /** @type {Sorter} */
        this.albums = new Sorter("year", [], false);
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
     * Gets artist by its name
     * @param {string} name - artist name
     * @returns {Artist|null} found artist or null
     */
    getArtistByName(name) {
        return this.allArtists.find(
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
        return this.allAlbums.find(
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
                s.title.toLowerCase().includes(q) ||
                s.artist.toLowerCase().includes(q) ||
                s.album.toLowerCase().includes(q)
            );
        });
        this.songs.set(queried);
    }

    searchAlbums(query) {
        const q = query.trim().toLowerCase();
        if (!q) {
            this.albums.set(this.allAlbums);
            return;
        }

        const filtered = this.allAlbums.filter((a) => {
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
            this.artists.set(this.allArtists);
            return;
        }

        const filtered = this.allArtists.filter((a) =>
            a.name.toLowerCase().includes(q),
        );
        this.artists.set(filtered);
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

        this.allAlbums = Array.from(albums.values());
        this.allAlbums.forEach((album) => album.sortByTrack());
        this.allAlbums.sort((a, b) => {
            const diff = b.year - a.year;
            if (diff !== 0) return diff;

            return a.name.localeCompare(b.name, undefined, {
                sensitivity: "accent",
            });
        });
        this.albums.set(this.allAlbums);

        this.allArtists = Array.from(artists.values());
        this.allArtists.sort((a, b) => a.name.localeCompare(b.name));
        this.allArtists.forEach((artist) => artist.sortAlbums());
        this.artists.set(this.allArtists);
    }
}
