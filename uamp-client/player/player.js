import {
    highlightAlbumSong,
    highlightArtistSong,
    highlightLibrary,
    highlightPlaylist,
} from "../ui/tables.js";
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
        this.playlist_stack = data.playlist_stack.map(
            (p) => new Playlist(p, library),
        );
        /** @type {number} */
        this.volume = data.volume;
        /** @type {boolean} */
        this.mute = data.mute;
        /** @type {string} */
        this.state = data.state;

        this.playerBar = document.querySelector("player-bar");
        this.playlistScreen = document.querySelector("playlist-screen");
    }

    /**
     * Creates new empty player.
     * @returns {Player} created player
     */
    static empty() {
        return new Player({
            volume: 0.69,
            mute: false,
            playlist: {
                songs: [],
                current: null,
                play_pos: null,
                on_end: null,
                add_policy: "None",
            },
            playlist_stack: [],
            state: "Paused",
        });
    }

    /**
     * Checks whether player is playing or not
     * @returns {boolean} true when playing, else false
     */
    isPlaying() {
        return this.state === "Playing";
    }

    /**
     * Gets currently playing song
     * @returns {Song|null} currently playing song if found
     */
    getPlaying = () => this.playlist.getPlaying();

    /**
     * Gets currently playing song ID
     * @returns {?number} ID of the currently playing song if found
     */
    getPlayingId = () => this.playlist.getPlayingId();

    /**
     * Gets playlist ID of the next playing song
     * @returns {number} playlist ID, 0 when no current set
     */
    getNextPId = () => this.playlist.getNextPId();

    /**
     * Sets the playback state and updates related UI elements.
     * @param {string} playback - playback state to set.
     */
    setPlayback(playback) {
        this.state = playback;
        this.playerBar.setPlaying(this.isPlaying());
    }

    /**
     * Sets the current song index in the playlist.
     * @param {?number} id - index of the current song in the playlist.
     */
    setCurrent(id) {
        this.playlist.current = id;
        this.highlightPlaying();
        this.playerBar.updateCurrent(this.getPlaying());
    }

    /**
     * Sets the volume level and updates the UI accordingly.
     * @param {number} volume - volume level to set (0 to 1).
     */
    setVolume(volume) {
        this.volume = volume;
        this.playerBar.updateVolume(volume, this.mute);
    }

    /**
     * Sets the mute state and updates the UI accordingly.
     * @param {boolean} mute - boolean indicating whether to mute or unmute.
     */
    setMute(mute) {
        this.mute = mute;
        this.playerBar.updateVolume(this.volume, mute);
    }

    /**
     * Gets playlist by its ID
     * @param {number} id - ID of the playlist (0 for current, 1+ for stack)
     * @returns {Playlist|null} playlist if found, else null
     */
    getPlaylist(id) {
        if (id === 0) return this.playlist;
        return this.playlist_stack[this.playlist_stack.length - id] || null;
    }

    /**
     * Pops playlists from the stack.
     * @param {number} cnt - number of playlists to pop, 0 means all
     * @returns {[Playlist|null, number]} removed playlist and popped count
     */
    popPlaylist(cnt = 1) {
        if (cnt === 0) cnt = this.playlist_stack.length;

        let prev = null;
        let removed = 0;
        while (cnt-- > 0 && this.playlist_stack.length > 0) {
            prev = this.playlist;
            this.playlist = this.playlist_stack.pop();

            this.playlistScreen.pop();
            removed += 1;
        }
        return [prev, removed];
    }

    /**
     * Removes given playlist from the stack.
     * @param {number} id - ID of the playlist to be removed
     */
    removePlaylist(id) {
        if (id === 0) {
            this.playlist = this.player.playlist_stack.pop();
        } else {
            this.playlist_stack.splice(this.playlist_stack.length - id, 1);
        }
    }

    /** Highlights currently playing song */
    highlightPlaying() {
        const id = this.getPlayingId();
        if (id === null) return;

        highlightLibrary(id);
        highlightPlaylist(this.playlist.current);
        highlightAlbumSong(id);
        highlightArtistSong(id);
    }
}
