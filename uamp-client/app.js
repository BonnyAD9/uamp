import Api from "./api.js";
import Duration from "./helper/duration.js";
import Timestamp from "./helper/timestamp.js";
import Library from "./library/library.js";
import Song from "./library/song.js";
import Player from "./player/player.js";
import Playlist from "./player/playlist.js";
import Config from "./settings.js";

export default class App {
    constructor() {
        /** @type {Library} */
        this.library = Library.empty();
        /** @type {Player} */
        this.player = Player.empty();
        /** @type {Timestamp|null} */
        this.position = null;
        /** @type {Config} */
        this.config = Config.empty();

        this.playlistTab = 0;

        this.playerBar = document.querySelector("player-bar");

        this.libraryScreen = document.querySelector("library-screen");
        this.albumsScreen = document.querySelector("albums-screen");
        this.artistsScreen = document.querySelector("artists-screen");
        this.playlistScreen = document.querySelector("playlist-screen");

        this.activeScreen = `#library`;
        this.activeOverlay = null;
        this.activeScreenArgs = {};
    }

    async init(data) {
        this.config.init(data.config).then(() => this.config.render());

        this.library = new Library(data.library);
        /** @type {Player} */
        this.player = new Player(data.player, this.library);
        /** @type {Timestamp} */
        this.position = data.position && Timestamp.from(data.position);
        this.playerBar.updateTimestamp(this.position);

        this.playlistTab = 0;

        this.libraryScreen.table.render();
        this.playlistScreen.table.render();
        this.playerBar.table.render();
    }

    /**
     * Sets the playback state and updates related UI elements.
     * @param {string} playback - playback state to set.
     */
    setPlayback(playback) {
        this.player.setPlayback(playback);
    }

    /**
     * Sets the current song index in the playlist and updates playlist table
     * @param {?number} id - index of the current song in the playlist
     */
    setCurrent(id) {
        this.player.setCurrent(id);
        this.playlistScreen.table.render();
    }

    /**
     * Sets the timestamp to given value and updates related UI elements.
     * @param {?Timestamp} timestamp - the timestamp to set.
     */
    setTimestamp(timestamp) {
        if (timestamp === null) {
            this.playerBar.updateTimestamp(null);
            this.position = null;
            return;
        }

        this.position = Timestamp.from(timestamp);
        const song = this.player.getPlaying();
        if (song) song.length = Duration.from(timestamp.total);
        this.playerBar.updateTimestamp(this.position);
    }

    /**
     * Sets the active playlist to the given one, updates UI.
     * @param {*} playlist - playlist to set as active.
     */
    setPlaylist(playlist) {
        this.player.playlist = new Playlist(playlist, this.library);
        if (this.playlistTab === 0) {
            this.displayPlaylist();
            this.createBarSongs();
        }
        this.player.highlightPlaying();
        this.playerBar.updateCurrent(this.player.getPlaying());
    }

    /**
     * Pushes active playlist to the stack and sets the new playlist. Updates
     * related UI elements.
     * @param {*} playlist - playlist to push onto the stack.
     */
    pushPlaylist(playlist) {
        this.player.playlist_stack.push(this.player.playlist);
        if (this.playlistTab !== 0) this.playlistTab += 1;

        this.playlistScreen.push();
        this.setPlaylist(playlist);
        this.playlistScreen.show(this.playlistTab);
    }

    /**
     * Pops playlists from the stack and sets it as the active playlist.
     * Updates related UI elements.
     * @param {number} cnt - number of playlists to pop from the stack.
     * @returns previous active playlist if it exists, otherwise null.
     */
    popPlaylist(cnt = 1) {
        const [prev, removed] = this.player.popPlaylist(cnt);
        this.playlistTab = Math.max(0, this.playlistTab - removed);

        this.playlistScreen.show(this.playlistTab);
        this.playerBar.updateCurrent(this.player.getPlaying());
        this.displayPlaylist();
        return prev;
    }

    /**
     * Removes given playlist and updates the UI.
     * @param {number} id - ID of the playlist to be removed
     */
    removePlaylist(id) {
        this.player.removePlaylist(id);
        this.playlistScreen.remove(id);

        const prev = this.playlistTab;
        if (this.playlistTab >= id)
            this.playlistTab = Math.max(0, this.playlistTab - 1);

        this.playlistScreen.show(this.playlistTab);

        if (prev !== this.playlistTab) {
            this.playerBar.updateCurrent(this.player.getPlaying());
            this.displayPlaylist();
        }
    }

    /**
     * Reorders the playlists based on the given indexes, updates related UI.
     * @param {number[]} indexes - reorder indexes containing all playlists
     */
    reorderPlaylists(indexes) {
        this.playlistScreen.reorder(indexes);
        if (this.playlistTab !== 0)
            this.playlistTab = indexes.indexOf(this.playlistTab);

        let order = indexes.reverse();
        const all = [
            ...this.player.playlist_stack.slice(),
            this.player.playlist,
        ];

        const len = order.length - 1;
        this.player.playlist = all[len - order[len]];
        this.player.playlist_stack = order
            .slice(0, -1)
            .map((i) => all[len - i]);

        this.playlistScreen.show(this.playlistTab);
        this.playerBar.updateCurrent(this.player.getPlaying());
        this.player.highlightPlaying();
    }

    /**
     * Sets active playlist tab and updates related UI elements.
     * @param {number} id - ID of the playlist tab
     */
    setPlaylistTab(id) {
        this.playlistTab = id;
        this.playlistScreen.show(id);
        this.displayPlaylist();
    }

    /**
     * Pushes temporary songs to the library.
     * @param {*} songs
     */
    pushTmpSongs(songs) {
        for (const [song, tid] of songs) {
            const id = -tid - 1;
            while (this.library.tmpSongs.length <= id) {
                const eid = -this.library.tmpSongs.length;
                this.library.tmpSongs.push(Song.empty(eid));
            }
            this.library.tmpSongs[id] = Song.from(tid, song);
        }
    }

    /** Displays songs with virtual scrolling. */
    displaySongs = () => this.libraryScreen.table.render();
    displayPlaylist = () => this.playlistScreen.table.render();
    createBarSongs = () => {
        this.playerBar.table.render();
        this.playerBar.updatePlaylistMask();
    };

    displayAlbums = () => this.albumsScreen.display(this.library.albums);
    displayArtists = () => this.artistsScreen.display(this.library.artists);

    searchLibrary = (e) => {
        this.library.searchLibrary(e.detail);
        this.libraryScreen.table.render();
    };
    searchAlbums = (e) => {
        this.library.searchAlbums(e.detail);
        this.displayAlbums();
    };
    searchArtists = (e) => {
        this.library.searchArtists(e.detail);
        this.displayArtists();
    };

    updateAll() {
        this.displayPlaylistStack();

        this.playerBar.updateCurrent(this.player.getPlaying());
        this.playerBar.setPlaying(this.player.isPlaying());
        this.playerBar.updateVolume(this.player.volume, this.player.mute);
    }

    displayPlaylistStack() {
        this.playlistScreen.displayStack(this.player.playlist_stack.length);
    }

    barPlaylistClick(e) {
        const item = e.target.closest(".item");
        if (!item) return;
        Api.ctrl(`pj=${item.dataset.index}`);
    }

    artistBarClick(e) {
        const artist = this.library.getArtistByName(e.target.textContent);
        this.navigateTo("artist-detail", { id: artist.id });

        // this.playerBar.toggleBar();
    }

    albumBarClick(artist, album) {
        const a = this.library.getAlbumByKey(artist, album);
        this.navigateTo("album-detail", { id: a.id });

        // this.playerBar.toggleBar();
    }

    /**
     * Navigates to the given screen, assumes it implements onNavigate
     * @param {string} target - target screen ID
     * @param {Object} args - target screen arguments
     * @param {boolean} pushHistory - whether to push history
     * @param {boolean} isOverlay - whether target is overlay only
     */
    navigateTo(target, args = {}, pushHistory = true, isOverlay = false) {
        if (isOverlay) {
            this.navigateToOverlay(target, args, pushHistory);
            return;
        }

        document.querySelector(this.activeScreen).classList.remove("active");
        this.activeScreen = `#${target}`;
        this.activeScreenArgs = args;

        const active = document.querySelector(this.activeScreen);
        active.classList.add("active");
        if (typeof active.onNavigate === "function") active.onNavigate(args);

        if (this.activeOverlay) {
            const overlay = document.querySelector(this.activeOverlay);
            overlay.classList.remove("active");
            this.activeOverlay = null;
        }

        if (pushHistory)
            history.pushState(
                { page: target, args, overlay: null, overlayArgs: null },
                "",
            );
    }

    /**
     * Navigates to the given overlay, only displays it and pushes to history
     * @param {string} target - target overlay ID
     * @param {Object} args - target overlay arguments
     */
    navigateToOverlay(target, args = {}, pushHistory = true) {
        this.activeOverlay = `#${target}`;
        const overlay = document.querySelector(this.activeOverlay);
        overlay.classList.add("active");
        if (typeof overlay.onNavigate === "function") overlay.onNavigate(args);

        if (pushHistory) {
            const cur = this.activeScreen.replace("#", "");
            history.pushState(
                {
                    page: cur,
                    args: this.activeScreenArgs,
                    overlay: target,
                    overlayArgs: args,
                },
                "",
            );
        }
    }

    /** Hides overlay by navigating to the active screen. */
    hideOverlay() {
        const target = this.activeScreen.replace("#", "");
        this.navigateTo(target, this.activeScreenArgs);
    }
}

const navs = document.querySelectorAll("nav p");

navs.forEach((item) => {
    item.addEventListener("click", () => {
        navs.forEach((p) => p.classList.remove("active"));
        item.classList.add("active");

        const targetId = item.dataset.screen;
        app.navigateTo(targetId);
    });
});

history.replaceState({ page: "library" }, "");
window.addEventListener("popstate", (e) => {
    const state = e.state || { page: "library", args: {}, overlay: null };
    app.navigateTo(state.page, state.args, false);

    if (state.overlay) {
        app.navigateToOverlay(state.overlay, state.overlayArgs, false);
    }

    navs.forEach((p) =>
        p.classList.toggle("active", p.dataset.screen === state.page),
    );
});
