import Duration from "../helper/duration.js";
import Album from "./album.js";

const UNDEF_YEAR = 2147483647;

const songTemplate = document.getElementById('song-template');
const barSongTemplate = document.getElementById('bar-song-template');

export default class Song {
    /**
     * Creates new song
     * @param {number} id
     * @param {string} path
     * @param {string|null} title
     * @param {[string]} artists
     * @param {string|null} album
     * @param {string|null} album_artist
     * @param {number|null} track
     * @param {number|null} disc
     * @param {number|null} year
     * @param {Duration|null} length
     * @param {[string]} genre
     * @param {boolean} deleted
     */
    constructor(
        id, path, title, artists, album, album_artist, track, disc, year, length, genres,
        deleted = false
    ) {
        /** @type {number} */
        this.id = id;
        /** @type {string} */
        this.path = path;
        /** @type {string|null} */
        this.title = title;
        /** @type {[string]} */
        this.artists = artists;
        /** @type {string|null} */
        this.album = album;
        /** @type {string|null} */
        this.album_artist = album_artist ?? (artists.length == 0 ? null : artists[0]);
        /** @type {number|null} */
        this.track = track;
        /** @type {number|null} */
        this.disc = disc;
        /** @type {number|null} */
        this.year = year;
        /** @type {Duration|null} */
        this.length = length;
        /** @type {[string]} */
        this.genres = genres;
        /** @type {boolean} */
        this.deleted = deleted;
    }

    static from(id, obj) {
        return new Song(
            id, obj.path, obj.title, obj.artists, obj.album, obj.album_artist,
            obj.track, obj.disc, obj.year, Duration.from(obj.length),
            obj.genres, obj.deleted
        );
    }

    static empty(id) {
        return new Song(
            id, '', '', '', '', 0, 0, 0, new Duration(0, 0), '', true
        );
    }

    /**
     * Gets songs release year, checks for not set year
     * @returns {string} songs release year
     */
    getYear() {
        return this.year === null ? '-' : `${this.year}`;
    }

    /**
     * Generates a table row with song details
     * @returns {HTMLTableRowElement} - generated song table row
     */
    getTableRow() {
        const cloned = songTemplate.content.cloneNode(true);
        const row = cloned.querySelector('tr');

        row.querySelector('img').src =
            Album.getCover(this.album_artist ?? "--", this.album ?? this.title ?? "--", 64);
        row.querySelector('.title').textContent = this.title ?? "-";
        row.querySelector('.author').textContent = this.artists.length == 0 ? "-" : this.artists.join(", ");
        row.querySelector('.album').textContent = this.album ?? "-";
        row.querySelector('.year').textContent = this.getYear();
        row.querySelector('.length').textContent = this.length?.format() ?? "-";
        row.querySelector('.genre').textContent = this.genres.length == 0 ? "-" : this.genres.join(", ");
        row.querySelector('.track').textContent = this.track ?? "-";
        row.querySelector('.disc').textContent = this.disc ?? "-";

        return row;
    }

    /**
     * Gets bar playlist song representation
     * @param {number} id 
     * @return {HTMLDivElement} bar playlist song
     */
    getBarRow(id) {
        const cloned = barSongTemplate.content.cloneNode(true);
        const item = cloned.querySelector('.item');

        item.querySelector('.id').textContent = id + 1;
        item.querySelector('.title').textContent = this.title;
        item.querySelector('.artist').textContent = this.artists.join(", ");

        return item;
    }

    /**
     * Gets uamp query for filtering the song
     * @returns {string} uamp query string
     */
    getQuery() {
        const s = (text) => text.replaceAll('/', '//');
        return `n=/${s(this.title ?? "")}/.p=/${s(this.album_artist ?? "")}/.a=/` +
            `${s(this.album ?? "")}/.t=${this.track ?? ""}.d=${this.disc ?? ""}.y=${this.year ?? ""}` +
            `.g=/${s(this.genres.length == 0 ? "" : this.genres[0])}/`;
    }
}
