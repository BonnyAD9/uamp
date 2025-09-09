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
     * @param {string} title
     * @param {string} artist
     * @param {string} album
     * @param {number} track
     * @param {number} disc
     * @param {number} year
     * @param {Duration} length
     * @param {string} genre
     * @param {boolean} deleted
     */
    constructor(
        id, path, title, artist, album, track, disc, year, length, genre,
        deleted = false
    ) {
        /** @type {number} */
        this.id = id;
        /** @type {string} */
        this.path = path;
        /** @type {string} */
        this.title = title;
        /** @type {string} */
        this.artist = artist;
        /** @type {string} */
        this.album = album;
        /** @type {number} */
        this.track = track;
        /** @type {number} */
        this.disc = disc;
        /** @type {number} */
        this.year = year;
        /** @type {Duration} */
        this.length = length;
        /** @type {string} */
        this.genre = genre;
        /** @type {boolean} */
        this.deleted = deleted;
    }

    static from(id, obj) {
        return new Song(
            id, obj.path, obj.title, obj.artist, obj.album, obj.track, obj.disc,
            obj.year, Duration.from(obj.length), obj.genre, obj.deleted
        );
    }

    static empty(id) {
        return new Song(
            id, '', '', '', '', 0, 0, UNDEF_YEAR, new Duration(0, 0), '', true
        );
    }

    /**
     * Gets songs release year, checks for not set year
     * @returns {string} songs release year
     */
    getYear() {
        return this.year == UNDEF_YEAR ? '-' : `${this.year}`;
    }

    /**
     * Generates a table row with song details
     * @returns {HTMLTableRowElement} - generated song table row
     */
    getTableRow() {
        const cloned = songTemplate.content.cloneNode(true);
        const row = cloned.querySelector('tr');

        row.querySelector('img').src =
            Album.getCover(this.artist, this.album, 64);
        row.querySelector('.title').textContent = this.title;
        row.querySelector('.author').textContent = this.artist;
        row.querySelector('.album').textContent = this.album;
        row.querySelector('.year').textContent = this.getYear();
        row.querySelector('.length').textContent = this.length.format();
        row.querySelector('.genre').textContent = this.genre;
        row.querySelector('.track').textContent = this.track;
        row.querySelector('.disc').textContent = this.disc;

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
        item.querySelector('.artist').textContent = this.artist;

        return item;
    }

    /**
     * Gets uamp query for filtering the song
     * @returns {string} uamp query string
     */
    getQuery() {
        const s = (text) => text.replaceAll('/', '//');
        return `n=/${s(this.title)}/.p=/${s(this.artist)}/.a=/` +
            `${s(this.album)}/.t=${this.track}.d=${this.disc}.y=${this.year}` +
            `.g=/${s(this.genre)}/`;
    }
}
