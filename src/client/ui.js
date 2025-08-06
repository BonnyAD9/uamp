const playBtn = document.getElementById('play');
/**
 * Updates the play button based on the playing state
 * @param {boolean} playing - whether player is playing or not
 */
function updatePlayBtn(playing) {
    const icon = playing ? 'pause.svg' : 'play.svg';
    playBtn.src = 'assets/svg/' + icon;
}

const songTitle = document.querySelector('.info .title h3');
const songArtist = document.querySelector('.info .title h4');
/**
 * Updates the currently playing song info
 * @param {?Song} song - currently playing song
 */
function updateCurrent(song) {
    if (song === null) {
        songTitle.textContent = 'Not Playing...';
        return;
    }

    songTitle.textContent = song.title;
    songArtist.textContent = song.artist;
}

const volumeSlider = document.getElementById('volumeSlider');
volumeSlider.addEventListener('input', () => {
    apiCtrl(`v=${volumeSlider.value / 100}`);
});

const volumeValue = document.getElementById('volumeValue');
const volumeIcon = document.querySelector('.volume img');
/**
 * Updates the volume UI elements
 * @param {number} volume - current playback volume
 * @param {boolean} mute - mute state
 */
function updateVolume(volume, mute = false) {
    const perVolume = Math.round(volume * 100);
    volumeSlider.value = perVolume;
    volumeValue.textContent = perVolume;

    const level = Math.ceil(volume * 4);
    let icon = `${mute ? 'no_' : ''}volume_${level}.svg`;
    volumeIcon.src = `assets/svg/${icon}`;
}

/**
 * Highlights song with with given index in the library
 * @param {?number} index - song index in the library
 */
function highlightLibrary(index) {
    const rows = document.querySelectorAll('#library .songs tbody tr');
    this.highlightPlaying(index, rows)
}

/**
 * Highlights song with given index in the playlist
 * @param {?number} index - song index in the playlist
 */
function highlightPlaylist(index) {
    const rows = document.querySelectorAll('#playlist #playingTable tbody tr');
    this.highlightPlaying(index, rows);
}

/**
 * Highlights song with given index in the given song rows
 * @param {?number} index - index of the song to highlight
 * @param {NodeListOf<HTMLTableRowElement>} rows - song table rows
 */
function highlightPlaying(index, rows) {
    for (let i = 0; i < rows.length; i++) {
        const row = rows[i];
        row.classList.remove('active');
        if (index === i)
            row.classList.add('active');
    }
}

const playlists = document.querySelector('#playlist .playlist-wrapper');
/**
 * Pushes playing table to the playlist stack, keeps the original playing table
 */
function pushPlaylist() {
    const playing = playlists.querySelector('#playingTable');

    const clone = playing.cloneNode(true);
    clone.removeAttribute('id');

    playlists.insertBefore(clone, playing.nextSibling);
}

/**
 * Pops playlist from the playlist stack, sets the playing table as well
 */
function popPlaylist() {
    const next = playlists.querySelector('.playlist-stack:not(#playingTable)');
    if (next === null) return;

    const playing = playlists.querySelector('#playingTable');
    playing.remove();

    next.id = 'playingTable';

    const rows = next.querySelectorAll('tbody tr');
    rows.forEach((row, i) => row.onClick = () => apiCtrl(`pj=${i}`));
}

/**
 * Removes row of the playing playlist based on the given row ID
 * @param {number} id - row ID to be removed
 */
function removePlaylistRow(id) {
    const rows = playlists.querySelectorAll('#playingTable tbody tr');
    if (rows.length <= id) return;

    if (rows[id].classList.contains('active') && rows[id].nextSibling !== null)
        rows[id].nextSibling.classList.add('active');
    rows[id].remove();
}

const playlistTabs = document.querySelector('#playlist .tabs');
const playlistTabsFiller = playlistTabs.querySelector('.filler');
/**
 * Adds new playlist tab to the end
 */
function pushPlaylistTab(i = null) {
    const tab = document.createElement('button');
    tab.classList.add('tab');

    const id = i ?? playlistTabs.querySelectorAll('.tab').length;
    tab.textContent = `-${id}`;
    tab.onclick = () => AppSingleton.get().setPlaylistTab(id);

    playlistTabs.insertBefore(tab, playlistTabsFiller);
}

/**
 * Removes the last playlist tab
 */
function popPlaylistTab() {
    const tabs = playlistTabs.querySelectorAll('.tab:not(#playingPlaylist)');
    if (tabs.length == 0) return;
    tabs[tabs.length - 1].remove();
}

/**
 * Displays a playlist based on its ID in the playlist stack
 * @param {number} id - ID of the playlist stack
 */
function showPlaylist(id) {
    const tabs = playlistTabs.querySelectorAll('.tab');
    const playlistStacks = playlists.querySelectorAll('.playlist-stack');

    for (let i = 0; i < tabs.length; i++) {
        const tab = tabs[i];
        const playlist = playlistStacks[i];

        tab.classList.remove('active');
        playlist.classList.remove('active');
        if (i === id) {
            tab.classList.add('active');
            playlist.classList.add('active');
        }
    }
}
