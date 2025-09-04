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
    displayAlbum, displayAlbums, displayArtist, playlists, playlistTabs,
    popPlaylist, popPlaylistTab, pushPlaylist, pushPlaylistTab,
    reorderPlaylists, showPlaylist, tableTemplate, updateCurrent, updatePlayBtn, updateVolume
} from "./ui.js";

const slider = document.querySelector('.bar .slider hr');

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

        this.playlistTab = 0;

        /** @type {?Album} */
        this.album = null;
        /** @type {?Artist} */
        this.artist = null;

        /** @type {{ start: number, end: number }} */
        this.songsBuffer = { start: 0, end: 0 };
        /** @type {{ start: number, end: number }} */
        this.playlistBuffer = { start: 0, end: 0 };
        /** @type {{ start: number, end: number }} */
        this.barBuffer = { start: 0, end: 0 };

        document.querySelector('#library').onscroll = () => this.updateSongs();
        document.querySelector('#playlist').onscroll =
            () => this.updatePlaylist();
    }

    static async init(data) {
        const config = await Config.init(data.config);
        return new App(data, config);
    }

    /**
     * Gets song from the library based on the given id
     * @param {number} id - id of the song
     * @returns {?Song} song when found, else null
     */
    getSong(id) {
        if (id < 0)
            return this.library.tmpSongs[-id - 1];
        return this.library.allSongs[id];
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
        if (this.playlistTab !== 0)
            this.playlistTab += 1;

        pushPlaylist();
        pushPlaylistTab();
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
        if (cnt === 0)
            cnt = this.player.playlist_stack.length;

        let prev = null;
        while (cnt-- > 0 && this.player.playlist_stack.length > 0) {
            prev = this.player.playlist;
            this.player.playlist = this.player.playlist_stack.pop();

            popPlaylist();
            popPlaylistTab();
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
            this.player.playlist
        ];

        const len = order.length - 1;
        this.player.playlist = all[len - order[len]];
        this.player.playlist_stack = order.slice(0, -1).map(i => all[len - i]);

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


    /** Displays songs in the library with virtual scrolling. */
    displaySongs() {
        const library = document.querySelector('#library');
        const table = library.querySelector('table.songs tbody');
        this.songsBuffer =
            this.genericDisplaySongs(library, table, this.library.songs);
    }

    displayPlaylist() {
        const container = document.querySelector('#playlist');
        const table = container.querySelector('.playlist-stack.active tbody');

        const songs = this.player.getPlaylist(this.playlistTab).songs;
        this.playlistBuffer = this.genericDisplaySongs(container, table, songs);
    }

    /** Updates visible songs in the library based on the scroll position. */
    updateSongs() {
        const library = document.querySelector('#library');
        const table = library.querySelector('table.songs tbody');
        this.songsBuffer = this.genericUpdateSongs(
            library, table, this.library.songs, this.songsBuffer
        );
    }

    updatePlaylist() {
        const playlist = document.querySelector('#playlist');
        const table = playlist.querySelector('.playlist-stack.active tbody');
        const songs = this.player.getPlaylist(this.playlistTab).songs;
        this.playlistBuffer = this.genericUpdateSongs(
            playlist, table, songs, this.playlistBuffer
        );
    }

    /**
     * Displays songs in the given containers table using virtual scrolling.
     * @param {HTMLElement} container - scrollable container with songs table
     * @param {HTMLElement} table - songs table body
     * @param {Song[]} songs - list of songs to display
     * @returns {{ start: number, end: number }} buffer boundaries
     */
    genericDisplaySongs(container, table, songs) {
        const current = this.player.getPlayingId();
        table.innerHTML = '';

        const top = document.createElement('tr');
        top.classList.add('spacer', 'spacer-top');
        table.appendChild(top);

        const bottom = document.createElement('tr');
        bottom.classList.add('spacer', 'spacer-bottom');
        table.appendChild(bottom);

        const { start, end } =
            this.#getBufferPos(container, songs.length, top, bottom);

        const fragment = document.createDocumentFragment();
        for (let i = start; i < end; i++)
            fragment.appendChild(this.#getRow(songs, i, current));
        top.after(fragment);

        return { start, end };
    }

    /**
     * TODO: playlist stack should highlight only their song
     * Updates songs table in the given container with virtual scrolling.
     * @param {HTMLElement} container - scrollable container with songs table
     * @param {HTMLElement} table - songs table body
     * @param {Song[]} songs - list of songs to display
     * @param {{ start: number, end: number }} oldBuffer - old buffer boundaries
     * @returns {{ start: number, end: number }} buffer boundaries
     */
    genericUpdateSongs(container, table, songs, oldBuffer) {
        const current = this.player.getPlayingId();

        const top = table.querySelector('tr.spacer-top');
        const bottom = table.querySelector('tr.spacer-bottom');
        const { start, end } =
            this.#getBufferPos(container, songs.length, top, bottom);

        for (let i = oldBuffer.start - 1; i >= start; i--)
            top.after(this.#getRow(songs, i, current));
        for (let i = oldBuffer.end; i < end; i++) {
            bottom.before(this.#getRow(songs, i, current));
        }

        const removeRow = row => {
            if (row && !row.classList.contains('spacer'))
                table.removeChild(row);
        }
        for (let i = oldBuffer.start; i < start; i++)
            removeRow(top.nextSibling);
        for (let i = oldBuffer.end; i > end; i--)
            removeRow(bottom.previousSibling);

        return { start, end };
    }

    createBarSongs() {
        const current = this.player.getPlayingId();
        const songs = this.player.playlist.songs;

        const container = document.querySelector('.bar .playlist');
        const table = container.querySelector('.songs');
        table.innerHTML = '';

        const top = document.createElement('div');
        top.classList.add('spacer', 'spacer-top');
        table.appendChild(top);

        const bottom = document.createElement('div');
        bottom.classList.add('spacer', 'spacer-bottom');
        table.appendChild(bottom);

        const { start, end } =
            this.#getBufferPos(container, songs.length, top, bottom, 32);

        const getItem = id => {
            const song = this.player.playlist.songs[id];
            const item = song.getBarRow(id);
            item.dataset.index = id;
            if (song.id === current)
                item.classList.add('active');
            return item;
        }

        const fragment = document.createDocumentFragment();
        for (let i = start; i < end; i++)
            fragment.appendChild(getItem(i));
        top.after(fragment);

        return { start, end };
    }

    updateBarSongs() {
        const current = this.player.getPlayingId();
        const playlist = document.querySelector('.bar .playlist');
        const list = playlist.querySelector('.songs');

        const top = list.querySelector('div.spacer-top');
        const bottom = list.querySelector('div.spacer-bottom');
        const { start, end } =
            this.#getBufferPos(playlist, this.player.playlist.songs.length,
                top, bottom, 32);

        for (let i = this.barBuffer.start - 1; i >= start; i--)
            top.after(this.#getItem(i, current));
        for (let i = this.barBuffer.end; i < end; i++) {
            bottom.before(this.#getItem(i, current));
        }

        const removeItem = row => {
            if (row && !row.classList.contains('spacer'))
                list.removeChild(row);
        }
        for (let i = this.barBuffer.start; i < start; i++)
            removeItem(top.nextSibling);
        for (let i = this.barBuffer.end; i > end; i--)
            removeItem(bottom.previousSibling);

        this.barBuffer = { start, end };
    }

    /**
     * Gets buffer position for the virtual scrolling and updates spacers
     * @param {HTMLElement} container - scrollable container
     * @param {number} songsCnt - total number of songs/rows
     * @param {HTMLTableRowElement} topSpacer - top spacer row
     * @param {HTMLTableRowElement} bottomSpacer - bottom spacer row
     * @returns {{ start: number, end: number }} buffer boundaries
     */
    #getBufferPos(container, songsCnt, topSpacer, bottomSpacer, row = 42) {
        const viewHeight = container.clientHeight;

        const visibleCnt = Math.ceil(viewHeight / row) + 1;
        const scrollTop = container.scrollTop;
        const start = Math.max(0, Math.floor(scrollTop / row) - 2);
        const end = Math.min(songsCnt, start + visibleCnt);

        topSpacer.style.height = `${start * row}px`;
        bottomSpacer.style.height =
            `${(songsCnt - end) * row}px`;
        return { start, end };
    }

    /**
     * Gets table row for the given song id
     * @param {Song[]} songs - list of songs to get the song from
     * @param {number} id - song id to get the row for
     * @param {number} current - song id of the currently playing song
     * @returns {HTMLTableRowElement} table row for the given song id
     */
    #getRow(songs, id, current) {
        const song = songs[id];
        const row = song.getTableRow();
        row.dataset.index = id;
        row.dataset.songId = song.id;
        if (song.id === current)
            row.classList.add('active');
        return row;
    }

    /**
     * Gets bar playlist item for the given song id
     * @param {number} id - song id to get the row for
     * @param {number} current - song id of the currently playing song
     * @returns {HTMLTableRowElement} table row for the given song id
     */
    #getItem(id, current) {
        const song = this.player.playlist.songs[id];
        const item = song.getBarRow(id);
        item.dataset.index = id;
        item.dataset.songId = song.id;
        if (song.id === current)
            item.classList.add('active');
        return item;
    }

    displayAlbums() {
        const albums = document.querySelector('#albums .list');
        displayAlbums(albums, this.library.albums);
    }

    displayArtists() {
        const artists = document.querySelector('#artists .songs tbody');
        artists.innerHTML = '';
        this.library.artists.forEach((artist, i) => {
            const row = artist.getTableRow();
            row.dataset.index = i;
            artists.appendChild(row);
        });
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
        if (this.radId !== null) {
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
        if (current > total)
            current = total;

        const percent = (current / total) * 100;
        slider.style.width = `${percent}%`;

        if (this.position !== null) {
            this.position.current.secs = Math.floor(current);
            this.position.current.nanos =
                Math.floor((current % 1) * 1_000_000_000);
        }
    }

    displayPlaylistStack() {
        playlistTabs.querySelectorAll('.tab:not(#playingPlaylist)')
            .forEach(tab => tab.remove());
        playlists.querySelectorAll('.playlist-stack')
            .forEach((table, i) => i !== 0 && table.remove());

        const len = this.player.playlist_stack.length;
        for (let i = 1; i <= len; i++) {
            const id = len - i;
            const playlist = this.player.playlist_stack[id];

            const cloned = this.getSongsTable(
                playlist.songs,
                playlist.current,
                (e) => this.playlistClick(e)
            );
            playlists.appendChild(cloned);

            const button = document.createElement('button');
            button.classList.add('tab');
            button.textContent = `-${i}`;
            button.onclick = () => this.setPlaylistTab(i);
            playlistTabs.appendChild(button);
        }
    }

    /**
     * Creates new songs table with the given songs
     * @param {Array} songs - array of songs.
     * @param {number} current - index of the current song.
     * @param {Function} onClick - optional click handler for each song.
     * @returns {HTMLTableElement}
     */
    getSongsTable(songs, current, onClick = (_) => { }) {
        const cloned = tableTemplate.content.cloneNode(true);
        const table = cloned.querySelector('table');
        table.classList.add('playlist-stack');

        const element = cloned.querySelector('tbody');
        element.addEventListener('click', e => onClick(e));
        element.innerHTML = '';

        for (let i = 0; i < songs.length; i++) {
            const song = songs[i];
            if (song === null || song.deleted === true) continue;

            const row = song.getTableRow();
            row.dataset.index = i;
            if (i === current)
                row.classList.add('active');
            element.appendChild(row);
        }

        return table;
    }

    libraryClick = e => this.genericSongClick(e, 'any');
    albumSongClick = e => this.genericSongClick(e, this.album.getQuery());
    artistSongClick = e => this.genericSongClick(e, this.artist.getQuery());

    genericSongClick(e, query) {
        const row = e.target.closest('tr');
        if (!row) return;

        const encodedQuery = encodeURIComponent(query);
        apiCtrl(`sp=${encodedQuery}&pj=${row.dataset.index}&pp=play`);
    }

    playlistClick(e) {
        const row = e.target.closest('tr');
        const table = row?.closest('table');
        if (!row || !table) return;

        const first =
            document.querySelector('#playlist .playlist-stack table');
        let cmd = '';
        if (table !== first) {
            cmd = `rps=${this.playlistTab}&`;
        }

        apiCtrl(`${cmd}pj=${row.dataset.index}&pp=play`);
    }

    barPlaylistClick(e) {
        const item = e.target.closest('.item');
        if (!item) return;
        apiCtrl(`pj=${item.dataset.index}`);
    }

    albumClick = (e) => this.genericAlbumClick(e, this.library.albums);
    albumArtistClick = (e) => this.genericAlbumClick(e, this.artist.albums);

    genericAlbumClick(e, albums) {
        const card = e.target.closest('.card');
        if (!card) return;

        this.album = albums[card.dataset.index];
        displayAlbum(this.album, this.player.getPlayingId());
        showScreen('album-detail');
    }

    artistClick(e) {
        const row = e.target.closest('tr');
        if (!row) return;

        const artist = this.library.artists[row.dataset.index];

        const album = e.target.closest('.albums-preview img');
        if (!album) {
            this.artist = artist;
            displayArtist(this.artist, this.player.getPlayingId());
            showScreen('artist-detail');
            return;
        }

        this.album = artist.albums[album.dataset.index];
        displayAlbum(this.album, this.player.getPlayingId());
        showScreen('album-detail');
    }
}

const navs = document.querySelectorAll('nav p');
const screens = document.querySelectorAll('.screen');

navs.forEach(item => {
    item.addEventListener('click', () => {
        navs.forEach(p => p.classList.remove('active'));
        item.classList.add('active');

        const targetId = item.dataset.screen;
        showScreen(targetId);
        if (targetId === 'playlist') {
            const app = AppSingleton.get();
            app.displayPlaylist();
        }
    });
});

function showScreen(target, pushHistory = true) {
    screens.forEach(s => s.classList.toggle('active', s.id === target));
    if (pushHistory)
        history.pushState({ page: target }, '');
}

history.replaceState({ page: 'library' }, '');
window.addEventListener('popstate', e => {
    const target = e.state?.page || 'library';
    showScreen(target, false);
    navs.forEach(
        p => p.classList.toggle('active', p.dataset.screen === target)
    );
});

const sliderTrack = document.querySelector('.bar .slider');
sliderTrack.addEventListener('click', e => {
    const rect = sliderTrack.getBoundingClientRect();
    const percent = (e.clientX - rect.left) / rect.width;

    const app = AppSingleton.get();
    const song = app.getPlaying();

    const pos = song.length.fromPercent(percent);
    apiCtrl(`seek=${pos.format()}`);
    slider.style.width = `${percent * 100}%`;
});

function apiCtrl(query) {
    return fetch(`/api/ctrl?${query}`)
        .then(res => {
            if (!res.ok) {
                throw new Error(`HTTP error! status: ${res.status}`);
            }
            return res.text();
        }).catch(error => {
            console.error('Fetch error:', error);
        });
}
window.apiCtrl = apiCtrl;
