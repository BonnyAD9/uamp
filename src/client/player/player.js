import {
    highlightAlbumSong, highlightArtistSong, highlightLibrary,
    highlightPlaylist, updateCurrent, updatePlayBtn, updateVolume
} from "../ui.js";
import Playlist from "./playlist.js";

export default class Player {
    /**
     * Creates new player
     * @param {object} data
     * @param {Library} library
     */
    constructor(data, library) {
        /** @type {Playlist} */
        this.playlist = new Playlist(data.playlist, library);
        /** @type {Playlist[]} */
        this.playlist_stack =
            data.playlist_stack.map(p => new Playlist(p, library));
        /** @type {number} */
        this.volume = data.volume;
        /** @type {boolean} */
        this.mute = data.mute;
        /** @type {string} */
        this.state = data.state;
    }

    /**
     * Checks whether player is playing or not
     * @returns {boolean} true when playing, else false
     */
    isPlaying() {
        return this.state === 'Playing';
    }

    /**
     * Gets currently playing song
     * @returns {?Song} currently playing song if found
     */
    getPlaying = () => this.playlist.getPlaying();

    /**
     * Gets currently playing song ID
     * @returns {?number} ID of the currently playing song if found
     */
    getPlayingId = () => this.playlist.getPlayingId();

    /**
     * Sets the playback state and updates related UI elements.
     * @param {string} playback - playback state to set.
     */
    setPlayback(playback) {
        this.state = playback;
        updatePlayBtn(this.isPlaying());
    }

    /**
     * Sets the current song index in the playlist.
     * @param {?number} id - index of the current song in the playlist.
     */
    setCurrent(id) {
        this.playlist.current = id;
        this.highlightPlaying();
        updateCurrent(this.getPlaying());
    }

    /**
     * Sets the volume level and updates the UI accordingly.
     * @param {number} volume - volume level to set (0 to 1).
     */
    setVolume(volume) {
        this.volume = volume;
        updateVolume(volume, this.mute);
    }

    /**
     * Sets the mute state and updates the UI accordingly.
     * @param {boolean} mute - boolean indicating whether to mute or unmute.
     */
    setMute(mute) {
        this.mute = mute;
        updateVolume(this.volume, mute);
    }

    /**
     * Gets playlist by its ID
     * @param {number} id - ID of the playlist (0 for current, 1+ for stack)
     * @returns {Playlist|null} playlist if found, else null
     */
    getPlaylist(id) {
        if (id === 0)
            return this.playlist;
        return this.playlist_stack[this.playlist_stack.length - id] || null;
    }

    /** Highlights currently playing song */
    highlightPlaying() {
        const id = this.getPlayingId();
        highlightLibrary(id);
        highlightPlaylist(id);
        highlightAlbumSong(id);
        highlightArtistSong(id);
    }
}
