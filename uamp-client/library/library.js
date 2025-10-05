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
        /** @type {Album[]} */
        this.allAlbums = [];
        /** @type {Artist[]} */
        this.allArtists = [];

        /** @type {Song[]} */
        this.tmpSongs = library.tmp_songs.map((s, i) => Song.from(-i - 1, s));

        /** @type {Songs} */
        this.songs = new Songs();
        /** @type {Album[]} */
        this.albums = [];
        /** @type {Artist[]} */
        this.artists = [];

        this.#generate();
        this.#filterLists();
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

    getArtistByName(name) {
        return this.allArtists.find(
            (a) => a.name.toLowerCase() === name.toLowerCase(),
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
            this.albums = this.allAlbums;
            return;
        }

        this.albums = this.allAlbums.filter((a) => {
            return (
                a.artist.toLowerCase().includes(q) ||
                a.name.toLowerCase().includes(q)
            );
        });
    }

    searchArtists(query) {
        const q = query.trim().toLowerCase();
        if (!q) {
            this.artists = this.allArtists;
            return;
        }

        this.artists = this.allArtists.filter((a) =>
            a.name.toLowerCase().includes(q),
        );
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
        this.albums = this.allAlbums;

        this.allArtists = Array.from(artists.values());
        this.allArtists.sort((a, b) => a.name.localeCompare(b.name));
        this.allArtists.forEach((artist) => artist.sortAlbums());
        this.artists = this.allArtists;
    }

    #filterLists() {
        const libSearch = document.getElementById("library-search");
        if (libSearch.value) {
            this.searchLibrary(libSearch.value);
        }

        const albumSearch = document.getElementById("albums-search");
        if (albumSearch.value) {
            this.searchAlbums(albumSearch.value);
        }

        const artistSearch = document.getElementById("artists-search");
        if (artistSearch.value) {
            this.searchArtists(artistSearch.value);
        }
    }
}
