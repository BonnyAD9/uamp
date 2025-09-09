import Duration from "../helper/duration.js";

export default class Playlist {
    /**
     * Creates new playlist
     * @param {object} data
     * @param {Library} library 
     */
    constructor(data, library) {
        /** @type {Song[]} */
        this.songs = Playlist.#parseSongs(data.songs, library);
        /** @type {?number} */
        this.current = data.current;
        /** @type {Duration} */
        this.play_pos = data.play_pos !== null ?
            Duration.from(data.play_pos) : null;
        /** @type {{ name: string, args: string[] }|null} */
        this.on_end = data.on_end;
        /** @type {string} */
        this.add_policy = data.add_policy;
    }

    /**
     * Gets currently playing song
     * @returns {?Song} currently playing song if found
     */
    getPlaying() {
        return this.current !== null ? this.songs[this.current] : null;
    }

    /**
     * Gets currently playing song ID
     * @returns {?number} ID of the currently playing song if found
     */
    getPlayingId() {
        return this.current !== null ? this.songs[this.current].id : null;
    }

    /** Parses the given list of song IDs into songs list */
    static #parseSongs(songs, library) {
        let parsed = [];
        for (const id of songs) {
            const song = library.getSong(id);
            if (song === null || song.deleted === true) continue;
            parsed.push(song);
        }
        return parsed;
    }
}
