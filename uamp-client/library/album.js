import Song from "./song.js";
import Sorter from "./sorter.js";

const albumTemplate = document.getElementById("album-template");

export default class Album {
    /**
     * Creates new album
     * @param {string} name
     * @param {string} artist
     * @param {number|null} year
     * @param {Song[]} songs
     */
    constructor(name, artist, year, songs = []) {
        /** @type {string} */
        this.name = name;
        /** @type {string} */
        this.artist = artist;
        /** @type {number|null} */
        this.year = year;
        /** @type {Sorter} */
        this.songs = new Sorter("track", songs);
    }

    static from(obj, allSongs) {
        const songs = obj.songs.map((s, _) => allSongs[s]);
        const year = songs[0]?.year ?? null;
        return new Album(obj.name, obj.artist, year, songs);
    }

    /**
     * Gets albums release year, checks for not set year
     * @returns {string} songs release year
     */
    getYear() {
        return this.year === null ? "-" : `${this.year}`;
    }

    /**
     * Generates an album details card
     * @returns {HTMLElement} - generated album card
     */
    getCard() {
        const cloned = albumTemplate.content.cloneNode(true);
        const card = cloned.querySelector(".card");

        card.querySelector("img").src = Album.getCover(this.artist, this.name);
        card.querySelector(".name").textContent = this.name;
        card.querySelector(".artist").textContent = this.artist;
        return card;
    }

    /**
     * Sorts albums songs by track number
     */
    sortByTrack() {
        this.songs.sortBy("track", true);
    }

    /**
     * Gets uamp query for filtering the album
     * @returns {string} uamp query string
     */
    getQuery() {
        const s = (text) => text.replaceAll("/", "//");
        return `p=/${s(this.artist)}/.a=/${s(this.name)}/@/t`;
    }

    /**
     * Gets the API URL to get the album cover
     * @param {string|null} artist
     * @param {string|null} album
     * @return {string} API URL
     */
    static getCover(artist, album, size = null) {
        let req =
            `/api/img?artist=${encodeURIComponent(artist ?? "--")}&album=` +
            `${encodeURIComponent(album ?? "--")}&or=` +
            encodeURIComponent("/app/assets/svg/img_placeholder.svg");
        if (size !== null) req += `&size=${size}`;
        return req;
    }
}
