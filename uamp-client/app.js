import Api from "./api.js";
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
    displayAlbum,
    displayAlbums,
    displayAlbumSongs,
    displayArtist,
    displayArtists,
    displayArtistSongs,
} from "./ui/pages.js";
import {
    displayAlbumSongsSort,
    displayArtistSongsSort,
    displayLibrarySort,
    displayPlaylistStack,
    removePlaylist,
    pushPlaylist,
    reorderPlaylists,
    showPlaylist,
} from "./ui/tables.js";
import VirtualTable from "./ui/virtual-table.js";

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

        this.playerBar = document.querySelector("player-bar");

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
            () => this.player.getPlaylist(this.playlistTab).current,
        );
        this.playlistTable.playlist().autoScrolling();
        this.barPlaylistTable = new VirtualTable(
            () => this.player.playlist.songs,
            ".bar .playlist",
            ".songs",
            () => this.player.playlist.current,
        );
        this.barPlaylistTable.playlist().list().centering().autoScrolling();
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

        /** @type {?Album} */
        this.album = null;
        /** @type {?Artist} */
        this.artist = null;

        this.libraryTable.render();
        this.playlistTable.render();
        this.barPlaylistTable.render();
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
        this.playlistTable.render();
        this.barPlaylistTable.render();
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
        const [prev, removed] = this.player.popPlaylist(cnt);
        this.playlistTab = Math.max(0, this.playlistTab - removed);

        showPlaylist(this.playlistTab);
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
        removePlaylist(id);

        const prev = this.playlistTab;
        if (this.playlistTab >= id)
            this.playlistTab = Math.max(0, this.playlistTab - 1);

        showPlaylist(this.playlistTab);

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
        this.playerBar.updateCurrent(this.player.getPlaying());
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
        this.playerBar.updatePlaylistMask();
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
        this.playerBar.updateTimestamp(null);
        this.displayPlaylistStack();

        this.playerBar.updateCurrent(this.player.getPlaying());
        this.playerBar.setPlaying(this.player.isPlaying());
        this.playerBar.updateVolume(this.player.volume, this.player.mute);
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

        Api.pushPlaylist(songs, row.dataset.index);
    }

    genericSongClick(e, query, sort = "") {
        const row = e.target.closest("tr");
        if (!row) return;

        const encQuery = encodeURIComponent(query);
        const sortQuery = sort === "" ? "" : `&sort=${sort}`;
        Api.ctrl(`sp=${encQuery}${sortQuery}&pj=${row.dataset.index}&pp=play`);
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

        Api.ctrl(`${cmd}pj=${row.dataset.index}&pp=play`);
    }

    barPlaylistClick(e) {
        const item = e.target.closest(".item");
        if (!item) return;
        Api.ctrl(`pj=${item.dataset.index}`);
    }

    albumClick = (e) => this.genericAlbumClick(e);
    albumArtistClick = (e) => this.genericAlbumClick(e);

    genericAlbumClick(e) {
        const card = e.target.closest(".card");
        if (!card) return;

        const albumScreen = document.querySelector("album-screen");
        albumScreen?.onNavigate({ id: card.dataset.index });
        showScreen("album-detail");
    }

    artistClick(e) {
        const row = e.target.closest("tr");
        if (!row) return;

        const artist = this.library.allArtists[row.dataset.index];

        const album = e.target.closest(".albums-preview img");
        if (!album) {
            this.artist = artist;
            displayArtist(this.artist, this.player.getPlayingId());
            showScreen("artist-detail");
            return;
        }

        this.album = this.library.allAlbums[album.dataset.index];
        displayAlbum(this.album, this.player.getPlayingId());
        showScreen("album-detail");
    }

    artistBarClick(e) {
        this.artist = this.library.getArtistByName(e.target.textContent);
        displayArtist(this.artist, this.player.getPlayingId());
        this.playerBar.toggleBar();
        showScreen("artist-detail");
    }

    albumBarClick(artist, album) {
        this.album = this.library.getAlbumByKey(artist, album);
        displayAlbum(this.album, this.player.getPlayingId());
        this.playerBar.toggleBar();
        showScreen("album-detail");
    }
}

const navs = document.querySelectorAll("nav p");

navs.forEach((item) => {
    item.addEventListener("click", () => {
        navs.forEach((p) => p.classList.remove("active"));
        item.classList.add("active");

        const targetId = item.dataset.screen;
        showScreen(targetId);
        if (targetId === "playlist") {
            app.displayPlaylist();
        }
    });
});

function showScreen(target, pushHistory = true) {
    document
        .querySelectorAll(".screen")
        .forEach((s) => s.classList.toggle("active", s.id === target));
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
