class Playlist {
    /**
     * Creates new playlist
     * @param {Song[]} songs
     * @param {?number} current
     * @param {Duration} play_pos
     * @param {{ name: string, args: string[] }|null} on_end
     * @param {string} add_policy
     */
    constructor(songs, current, play_pos, on_end, add_policy) {
        /** @type {Song[]} */
        this.songs = songs;
        /** @type {?number} */
        this.current = current;
        /** @type {Duration} */
        this.play_pos = play_pos;
        /** @type {{ name: string, args: string[] }|null} */
        this.on_end = on_end;
        /** @type {string} */
        this.add_policy = add_policy;
    }

    /**
     * Creates playlist from object
     * @param {*} obj - object to create playlist from
     * @param {(object) => Song[]} parsePlaylist - function to parse playlist
     * @returns {Playlist} - created playlist
     */
    static from(obj, parsePlaylist) {
        return new Playlist(
            parsePlaylist(obj.songs),
            obj.current,
            obj.play_pos !== null ? Duration.from(obj.play_pos) : null,
            obj.on_end,
            obj.add_policy
        );
    }
}

class Player {
    /**
     * Creates new player
     * @param {Playlist} playlist
     * @param {Playlist[]} playlist_stack
     * @param {number} volume
     * @param {boolean} mute
     * @param {string} state
     */
    constructor(playlist, playlist_stack, volume, mute, state) {
        /** @type {Playlist} */
        this.playlist = playlist;
        /** @type {Playlist[]} */
        this.playlist_stack = playlist_stack;
        /** @type {number} */
        this.volume = volume;
        /** @type {boolean} */
        this.mute = mute;
        /** @type {string} */
        this.state = state;
    }

    /**
     * Creates player from object
     * @param {*} obj - object to create player from
     * @param {(object) => Song[]} parsePlaylist - function to parse playlist
     * @returns {Player} - created player
     */
    static from(obj, parsePlaylist) {
        return new Player(
            Playlist.from(obj.playlist, parsePlaylist),
            obj.playlist_stack.map(p => Playlist.from(p, parsePlaylist)),
            obj.volume,
            obj.mute,
            obj.state
        );
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
    getPlaying() {
        const playing = this.playlist.current;
        return playing !== null ? this.playlist.songs[playing] : null;
    }

    /**
     * Gets currently playing song ID
     * @returns {?number} ID of the currently playing song if found
     */
    getPlayingId() {
        const playing = this.playlist.current;
        return playing !== null ? this.playlist.songs[playing].id : null;
    }

    /**
     * Sets the playback state and updates related UI elements.
     * @param {string} playback - playback state to set.
     */
    setPlayback(playback) {
        this.state = playback;
        updatePlayBtn(this.isPlaying());
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
        return this.playlist_stack[id - 1] || null;
    }
}
