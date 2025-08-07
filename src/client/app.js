const songsElement = document.querySelector('#library .songs tbody');
const tableTemplate = document.getElementById('songs-template');
const slider = document.querySelector('.bar .slider hr');

class App {
    constructor(data) {
        this.library = {
            songs: data.library.songs.map(Song.from),
            tmp_songs: data.library.tmp_songs.map(Song.from),
        };
        this.player = data.player;
        this.position = data.position && Timestamp.from(data.position);

        this.lastUpdate = performance.now();
        this.rafId = null;

        this.playlistTab = 0;

        this.albums = [];
        this.artists = [];
        this.album = null;
        this.artist = null;

        this.generateLibraryData();
    }

    /**
     * Checks whether player is playing or not
     * @returns {boolean} true when playing, else false
     */
    isPlaying() {
        return this.player.state === 'Playing';
    }

    /**
     * Gets currently playing song
     * @returns {?Song} playing song if found
     */
    getPlaying() {
        const playing = this.player.playlist.current;
        if (playing === null) return null;

        const current = this.player.playlist.songs[playing];
        return this.library.songs[current];
    }

    /**
     * Sets the playback state and updates related UI elements.
     * @param {string} playback - playback state to set.
     */
    setPlayback(playback) {
        this.player.state = playback;
        this.handleSongProgress();
        updatePlayBtn(this.isPlaying());
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
        this.getPlaying().length = Duration.from(timestamp.total);
        this.displayProgress(0);
    }

    /**
     * Sets the current song index in the playlist.
     * @param {?number} id - index of the current song in the playlist.
     */
    setCurrent(id) {
        this.player.playlist.current = id;
        updateCurrent(this.getPlaying());
        this.highlightLibrary();
        this.highlightPlaylist();
    }

    /**
     * Sets the volume level and updates the UI accordingly.
     * @param {number} volume - volume level to set (0 to 1).
     */
    setVolume(volume) {
        this.player.volume = volume;
        updateVolume(volume, this.player.mute);
    }

    /**
     * Sets the mute state and updates the UI accordingly.
     * @param {boolean} mute - boolean indicating whether to mute or unmute.
     */
    setMute(mute) {
        this.player.mute = mute;
        updateVolume(this.player.volume, mute);
    }

    /**
     * Sets the active playlist to the given one, updates UI.
     * @param {*} playlist - playlist to set as active.
     */
    setPlaylist(playlist) {
        this.player.playlist = playlist;
        this.displayPlaylist();
        this.highlightLibrary();
        updateCurrent(this.getPlaying());
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
        updateCurrent(this.getPlaying());
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
        updateCurrent(this.getPlaying());
        this.highlightLibrary();
    }

    /**
     * Sets active playlist tab and updates related UI elements.
     * @param {number} id - ID of the playlist tab
     */
    setPlaylistTab(id) {
        this.playlistTab = id;
        showPlaylist(id);
    }


    displaySongs() {
        const playing = this.player.playlist.current;
        const current =
            playing !== null ? this.player.playlist.songs[playing] : null;

        songsElement.innerHTML = '';
        for (let i = 0; i < this.library.songs.length; i++) {
            const song = this.library.songs[i];
            if (song.deleted === true) continue;

            const row = song.getTableRow();
            row.addEventListener(
                'click',
                () => {
                    const play = this.isPlaying() ? 'play' : 'pause';
                    const push = encodeURIComponent(song.getQuery());
                    apiCtrl(`push=${push}&pp=${play}`);
                }
            );
            if (i === current)
                row.classList.add('active');
            songsElement.appendChild(row);
        }
    }

    displayPlaylist() {
        const playing = this.player.playlist.current;
        const playlistElement =
            document.querySelector('#playlist .playlist-stack tbody');
        playlistElement.innerHTML = '';
        for (let i = 0; i < this.player.playlist.songs.length; i++) {
            const song = this.library.songs[this.player.playlist.songs[i]];
            if (song.deleted === true) continue;

            const row = song.getTableRow();
            row.dataset.index = i;
            if (i === playing)
                row.classList.add('active');
            playlistElement.appendChild(row);
        }
    }

    displayAlbums() {
        const albums = document.querySelector('#albums .list');
        displayAlbums(albums, this.albums);
    }

    displayArtists() {
        const artists = document.querySelector('#artists .songs tbody');
        artists.innerHTML = '';
        this.artists.forEach((artist, i) => {
            const row = artist.getTableRow();
            row.dataset.index = i;
            artists.appendChild(row);
        });
    }

    /**
     * Highlights currently playing song in the library
     */
    highlightLibrary() {
        const playing = this.player.playlist.current;
        const current =
            playing !== null ? this.player.playlist.songs[playing] : null;
        highlightLibrary(current);
    }

    /**
     * Highlights currently playing song in the playlist
     */
    highlightPlaylist() {
        highlightPlaylist(this.player.playlist.current);
    }

    updateAll() {
        this.displayProgress(0);
        this.handleSongProgress();
        this.displayPlaylistStack();

        updateCurrent(this.getPlaying());
        updatePlayBtn(this.isPlaying());
        updateVolume(this.player.volume, this.player.mute);
    }

    /**
     * Handles song progress bar updates
     */
    handleSongProgress() {
        if (this.isPlaying()) {
            this.lastUpdate = performance.now();
            this.stopProgress();
            this.rafId = requestAnimationFrame(() => this.updateProgressBar());
        } else {
            this.stopProgress();
        }
    }

    /**
     * Stops the song progres bar updates
     */
    stopProgress() {
        if (this.radId !== null) {
            cancelAnimationFrame(this.rafId);
            this.rafId = null;
        }
    }

    /**
     * Updates progress bar based on delta time
     */
    updateProgressBar() {
        if (!this.isPlaying()) return;

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
        const total = this.getPlaying().length.toSecs();

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

    getDurationSecs(duration) {
        return duration.secs + duration.nanos / 1_000_000_000;
    }

    displayPlaylistStack() {
        playlistTabs.querySelectorAll('.tab:not(#playingPlaylist)')
            .forEach(tab => tab.remove());
        playlists.querySelectorAll('.playlist-stack')
            .forEach((table, i) => i !== 0 && table.remove());

        const filler = playlistTabs.querySelector('.filler');
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
            playlistTabs.insertBefore(button, filler);
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

        const element = cloned.querySelector('tbody');
        element.addEventListener('click', e => onClick(e));
        element.innerHTML = '';

        for (let i = 0; i < songs.length; i++) {
            const song = this.library.songs[songs[i]];
            if (song.deleted === true) continue;

            const row = song.getTableRow();
            row.dataset.index = i;
            if (i === current)
                row.classList.add('active');
            element.appendChild(row);
        }

        return cloned;
    }

    playlistClick(e) {
        const row = e.target.closest('tr');
        const table = row?.closest('table');
        if (!row || !table) return;

        const first =
            document.querySelector('#playlist .playlist-stack');
        let cmd = '';
        if (table !== first) {
            cmd = `rps=${this.playlistTab}&`;
        }

        const play = this.isPlaying() ? 'play' : 'pause';
        apiCtrl(`${cmd}pj=${row.dataset.index}&pp=${play}`);
    }

    albumClick = (e) => this.genericAlbumClick(e, this.albums);
    albumArtistClick = (e) => this.genericAlbumClick(e, this.artist.albums);

    genericAlbumClick(e, albums) {
        const card = e.target.closest('.card');
        if (!card) return;

        this.album = albums[card.dataset.index];
        displayAlbum(this.album);
        showScreen('album-detail');
    }

    artistClick(e) {
        const row = e.target.closest('tr');
        if (!row) return;

        const artist = this.artists[row.dataset.index];

        const album = e.target.closest('.albums-preview img');
        if (!album) {
            this.artist = artist;
            displayArtist(this.artist);
            showScreen('artist-detail');
            return;
        }

        this.album = artist.albums[album.dataset.index];
        displayAlbum(this.album);
        showScreen('album-detail');
    }

    generateLibraryData() {
        const albums = new Map();
        const artists = new Map();

        for (const song of this.library.songs) {
            if (song.deleted) continue;

            const artistKey = song.artist.trim().toLowerCase();
            if (!artists.has(artistKey)) {
                artists.set(artistKey, new Artist(song.artist));
            }

            const artist = artists.get(artistKey);
            artist.songs.push(song);

            const albumKey = `${song.album.trim().toLowerCase()}::${artistKey}`;
            if (!albums.has(albumKey)) {
                albums.set(
                    albumKey,
                    new Album(song.album, song.artist, song.year)
                );
                artist.albums.push(albums.get(albumKey));
            }
            albums.get(albumKey).songs.push(song);
        }

        this.albums = Array.from(albums.values());
        this.albums.forEach(album => album.sortByTrack());

        this.artists = Array.from(artists.values());
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
    });
});

function showScreen(target) {
    for (let screen of screens) {
        screen.classList.remove('active');
        if (screen.id == target) {
            screen.classList.add('active');
        }
    }
}

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
