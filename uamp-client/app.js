import Duration from "./helper/duration.js";
import Timestamp from "./helper/timestamp.js";
import Album from "./library/album.js";
import Artist from "./library/artist.js";
import Library from "./library/library.js";
import Song from "./library/song.js";
import Player from "./player/player.js";
import Playlist from "./player/playlist.js";
import Config from "./settings.js";
import {
    toggleBar,
    updateCurrent,
    updatePlayBtn,
    updatePlaylistMask,
    updateTimestamp,
    updateVolume,
} from "./ui/bar.js";
import {
    displayAlbum,
    displayAlbums,
    displayAlbumSongs,
    displayArtist,
    displayArtists,
    displayArtistSongs,
} from "./ui/pages.js";
import {
    displayAlbumSongsSort,
    displayAlbumsSort,
    displayArtistSongsSort,
    displayArtistsSort,
    displayLibrarySort,
    displayPlaylistStack,
    popPlaylist,
    pushPlaylist,
    reorderPlaylists,
    showPlaylist,
} from "./ui/tables.js";
import VirtualTable from "./ui/virtual_table.js";

const slider = document.querySelector(".bar .slider hr");

export default class App {
    /**
     * @param {object} data
     * @param {Config} config
     */
    constructor(data, config) {
        this.library = new Library(data.library);
        /** @type {Player} */
        this.player = new Player(data.player, this.library);
        /** @type {Timestamp} */
        this.position = data.position && Timestamp.from(data.position);
        /** @type {Config} */
        this.config = config;
        this.config.render();

        this.lastUpdate = performance.now();
        this.rafId = null;

        this.searchTimeout = null;

        this.playlistTab = 0;

        /** @type {?Album} */
        this.album = null;
        /** @type {?Artist} */
        this.artist = null;

        this.libraryTable = new VirtualTable(
            () => this.library.songs.get(),
            "#library",
            ".songs tbody",
            () => this.player.getPlayingId(),
        );
        this.playlistTable = new VirtualTable(
            () => this.player.getPlaylist(this.playlistTab).songs,
            "#playlist",
            ".playlist-stack.active tbody",
            () => this.player.getPlaylist(this.playlistTab).getPlayingId(),
            true,
            true,
        );
        this.barPlaylistTable = new VirtualTable(
            () => this.player.playlist.songs,
            ".bar .playlist",
            ".songs",
            () => this.player.getPlayingId(),
            false,
            true,
        );
    }

    static async init(data) {
        const config = await Config.init(data.config);
        return new App(data, config);
    }

    /**
     * Sets the playback state and updates related UI elements.
     * @param {string} playback - playback state to set.
     */
    setPlayback(playback) {
        this.player.setPlayback(playback);
        this.handleSongProgress();
    }

    /**
     * Sets the timestamp to given value and updates related UI elements.
     * @param {?Timestamp} timestamp - the timestamp to set.
     */
    setTimestamp(timestamp) {
        if (timestamp === null) {
            this.position = null;
            return;
        }

        this.position = Timestamp.from(timestamp);
        this.player.getPlaying().length = Duration.from(timestamp.total);
        this.displayProgress(0);
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
        updateCurrent(this.player.getPlaying());
    }

    /**
     * Pushes active playlist to the stack and sets the new playlist. Updates
     * related UI elements.
     * @param {*} playlist - playlist to push onto the stack.
     */
    pushPlaylist(playlist) {
        this.player.playlist_stack.push(this.player.playlist);
        if (this.playlistTab !== 0) this.playlistTab += 1;

        pushPlaylist();
        this.setPlaylist(playlist);
        showPlaylist(this.playlistTab);
    }

    /**
     * Pops playlists from the stack and sets it as the active playlist. Updates
     * related UI elements.
     * @param {number} cnt - number of playlists to pop from the stack.
     * @returns previous active playlist if it exists, otherwise null.
     */
    popPlaylist(cnt = 1) {
        if (cnt === 0) cnt = this.player.playlist_stack.length;

        let prev = null;
        while (cnt-- > 0 && this.player.playlist_stack.length > 0) {
            prev = this.player.playlist;
            this.player.playlist = this.player.playlist_stack.pop();

            popPlaylist();
            this.playlistTab -= 1;
        }

        this.playlistTab = Math.max(0, this.playlistTab);
        showPlaylist(this.playlistTab);
        updateCurrent(this.player.getPlaying());
        this.displayPlaylist();
        return prev;
    }

    /**
     * Reorders the playlists based on the given indexes, updates related UI.
     * @param {number[]} indexes - reorder indexes containing all playlists
     */
    reorderPlaylists(indexes) {
        reorderPlaylists(indexes);
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

        showPlaylist(this.playlistTab);
        updateCurrent(this.player.getPlaying());
        this.player.highlightPlaying();
    }

    /**
     * Sets active playlist tab and updates related UI elements.
     * @param {number} id - ID of the playlist tab
     */
    setPlaylistTab(id) {
        this.playlistTab = id;
        showPlaylist(id);
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
    displaySongs = () => this.libraryTable.render();
    displayPlaylist = () => this.playlistTable.render();
    createBarSongs = () => {
        this.barPlaylistTable.render();
        updatePlaylistMask();
    };

    displayAlbums = () => displayAlbums(this.library.albums);
    displayArtists = () => displayArtists(this.library.artists);

    sortSongs = (key) => {
        this.library.songs.toggleSort(key);
        this.libraryTable.render();
        displayLibrarySort(
            this.library.songs.key,
            this.library.songs.ascending,
        );
    };
    sortAlbumSongs = (key) => {
        if (!this.album) return;
        this.album.songs.toggleSort(key);
        displayAlbumSongs(this.album, this.player.getPlayingId());
        displayAlbumSongsSort(this.album.songs.key, this.album.songs.ascending);
    };
    sortArtistSongs = (key) => {
        if (!this.artist) return;
        this.artist.songs.toggleSort(key);
        displayArtistSongs(this.artist, this.player.getPlayingId());
        displayArtistSongsSort(
            this.artist.songs.key,
            this.artist.songs.ascending,
        );
    };

    sortAlbums = (key) => {
        this.library.albums.toggleSort(key);
        this.displayAlbums();
    };

    sortArtists = (key) => {
        this.library.artists.toggleSort(key);
        this.displayArtists();
    };

    searchLibrary = this.searchDebounce((e) => {
        this.library.searchLibrary(e.target.value);
        this.libraryTable.render();
    });
    searchAlbums = this.searchDebounce((e) => {
        this.library.searchAlbums(e.target.value);
        this.displayAlbums();
    });
    searchArtists = this.searchDebounce((e) => {
        this.library.searchArtists(e.target.value);
        this.displayArtists();
    });
    searchDebounce(searchFn, delay = 100) {
        return (e) => {
            clearTimeout(this.searchTimeout);
            this.searchTimeout = setTimeout(() => searchFn(e), delay);
        };
    }

    updateAll() {
        this.displayProgress(0);
        this.handleSongProgress();
        this.displayPlaylistStack();

        updateCurrent(this.player.getPlaying());
        updatePlayBtn(this.player.isPlaying());
        updateVolume(this.player.volume, this.player.mute);
    }

    /** Handles song progress bar updates */
    handleSongProgress() {
        if (this.player.isPlaying()) {
            this.lastUpdate = performance.now();
            this.stopProgress();
            this.rafId = requestAnimationFrame(() => this.updateProgressBar());
        } else {
            this.stopProgress();
        }
    }

    /** Stops the song progres bar updates */
    stopProgress() {
        if (this.rafId !== null) {
            cancelAnimationFrame(this.rafId);
            this.rafId = null;
        }
    }

    /** Updates progress bar based on delta time */
    updateProgressBar() {
        if (!this.player.isPlaying()) return;

        const now = performance.now();
        const delta = (now - this.lastUpdate) / 1000;
        this.lastUpdate = now;

        this.displayProgress(delta);
        this.rafId = requestAnimationFrame(() => this.updateProgressBar());
    }

    /**
     * Updates progress bar
     * @param {number} delta - optional delta time
     */
    displayProgress(delta = 0) {
        let current = 0 + delta;
        if (this.position !== null)
            current = this.position.current.toSecs() + delta;

        const playing = this.player.getPlaying();
        if (playing === null) return;

        const total = playing.length.toSecs();
        if (current > total) current = total;

        const percent = (current / total) * 100;
        slider.style.width = `${percent}%`;

        if (this.position !== null) {
            this.position.current.secs = Math.floor(current);
            this.position.current.nanos = Math.floor(
                (current % 1) * 1_000_000_000,
            );
            updateTimestamp(this.position.current);
        }
    }

    displayPlaylistStack() {
        displayPlaylistStack(this.player.playlist_stack.length);
    }

    libraryClick = (e) => this.songClickHandler(e, this.library.songs.get());
    albumSongClick = (e) => this.songClickHandler(e, this.album.songs.get());
    artistSongClick = (e) => this.songClickHandler(e, this.artist.songs.get());

    songClickHandler(e, songs) {
        const row = e.target.closest("tr");
        if (!row) return;

        apiPushPlaylist(songs, row.dataset.index);
    }

    genericSongClick(e, query, sort = "") {
        const row = e.target.closest("tr");
        if (!row) return;

        const encQuery = encodeURIComponent(query);
        const sortQuery = sort === "" ? "" : `&sort=${sort}`;
        apiCtrl(`sp=${encQuery}${sortQuery}&pj=${row.dataset.index}&pp=play`);
    }

    playlistClick(e) {
        const row = e.target.closest("tr");
        const table = row?.closest("table");
        if (!row || !table) return;

        const first = document.querySelector("#playlist .playlist-stack table");
        let cmd = "";
        if (table !== first) {
            cmd = `rps=${this.playlistTab}&`;
        }

        apiCtrl(`${cmd}pj=${row.dataset.index}&pp=play`);
    }

    barPlaylistClick(e) {
        const item = e.target.closest(".item");
        if (!item) return;
        apiCtrl(`pj=${item.dataset.index}`);
    }

    albumClick = (e) => this.genericAlbumClick(e, this.library.albums.get());
    albumArtistClick = (e) => this.genericAlbumClick(e, this.artist.albums);

    genericAlbumClick(e, albums) {
        const card = e.target.closest(".card");
        if (!card) return;

        this.album = albums[card.dataset.index];
        displayAlbum(this.album, this.player.getPlayingId());
        showScreen("album-detail");
    }

    artistClick(e) {
        const row = e.target.closest("tr");
        if (!row) return;

        const artist = this.library.artists.get()[row.dataset.index];

        const album = e.target.closest(".albums-preview img");
        if (!album) {
            this.artist = artist;
            displayArtist(this.artist, this.player.getPlayingId());
            showScreen("artist-detail");
            return;
        }

        this.album = artist.albums[album.dataset.index];
        displayAlbum(this.album, this.player.getPlayingId());
        showScreen("album-detail");
    }

    artistBarClick(e) {
        this.artist = this.library.getArtistByName(e.target.textContent);
        displayArtist(this.artist, this.player.getPlayingId());
        toggleBar();
        showScreen("artist-detail");
    }

    albumBarClick(artist, album) {
        this.album = this.library.getAlbumByKey(artist, album);
        displayAlbum(this.album, this.player.getPlayingId());
        toggleBar();
        showScreen("album-detail");
    }
}

const navs = document.querySelectorAll("nav p");
const screens = document.querySelectorAll(".screen");

navs.forEach((item) => {
    item.addEventListener("click", () => {
        navs.forEach((p) => p.classList.remove("active"));
        item.classList.add("active");

        const targetId = item.dataset.screen;
        showScreen(targetId);
        if (targetId === "playlist") {
            const app = AppSingleton.get();
            app.displayPlaylist();
        }
    });
});

function showScreen(target, pushHistory = true) {
    screens.forEach((s) => s.classList.toggle("active", s.id === target));
    if (pushHistory) history.pushState({ page: target }, "");
}

history.replaceState({ page: "library" }, "");
window.addEventListener("popstate", (e) => {
    const target = e.state?.page || "library";
    showScreen(target, false);
    navs.forEach((p) =>
        p.classList.toggle("active", p.dataset.screen === target),
    );
});

const sliderTrack = document.querySelector(".bar .slider");
sliderTrack.addEventListener("click", (e) => {
    const rect = sliderTrack.getBoundingClientRect();
    const percent = (e.clientX - rect.left) / rect.width;

    const app = AppSingleton.get();
    const song = app.player.getPlaying();

    const pos = song.length.fromPercent(percent);
    apiCtrl(`seek=${pos.format()}`);
    slider.style.width = `${percent * 100}%`;
});

/**
 * Sends an API request to push playlist
 * @param {Song[]} songs - songs to be pushed to the playlist
 * @param {number} pos - playing song in the playlist
 */
async function apiPushPlaylist(songs, pos) {
    const data = {
        songs: songs.map((s) => s.id),
        position: Number(pos),
        play: true,
    };
    return fetch("/api/ctrl", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ SetPlaylist: data }),
    });
}

async function apiCtrl(query) {
    return fetch(`/api/ctrl?${query}`)
        .then((res) => {
            if (!res.ok) {
                throw new Error(`HTTP error! status: ${res.status}`);
            }
            return res.text();
        })
        .catch((error) => {
            console.error("Fetch error:", error);
        });
}
window.apiCtrl = apiCtrl;
