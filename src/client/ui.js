const pausePlayP1 = document.getElementById("from_pause_to_play_p1");
const playPauseP1 = document.getElementById("from_play_to_pause_p1");
const pausePlayP2 = document.getElementById("from_pause_to_play_p2");
const playPauseP2 = document.getElementById("from_play_to_pause_p2");
/**
 * Updates the play button based on the playing state
 * @param {boolean} playing - whether player is playing or not
 */
function updatePlayBtn(playing) {
    if (playing) {
        playPauseP1.beginElement();
        playPauseP2.beginElement();
    } else {
        pausePlayP1.beginElement();
        pausePlayP2.beginElement();
    }
}

const songIcon = document.querySelector('.bar .info .info-pic img');
const songTitle = document.querySelector('.bar .info .title h3');
const songArtist = document.querySelector('.bar .info .title h4');
const barBackdrop = document.querySelector('.bar .backdrop');
/**
 * Updates the currently playing song info
 * @param {?Song} song - currently playing song
 */
function updateCurrent(song) {
    if (song === null) {
        songTitle.textContent = 'Not Playing...';
        songArtist.textContent = '';
        return;
    }

    songIcon.src = Album.getCover(song.artist, song.album);
    barBackdrop.src = Album.getCover(song.artist, song.album, 64);
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
    const table = document.querySelector('#playlist .playlist-stack tbody');
    const rows = table.querySelectorAll('tr');
    this.highlightPlaying(index, rows);
}

const barPlaylist = document.querySelector('.bar .playlist .songs');
/**
 * Highlights songs with given index in the playlist
 * @param {?number} index - song index in the playlist
 */
function highlightBarPlaylist(index) {
    const items = barPlaylist.querySelectorAll('.item');
    this.highlightPlaying(index, items);
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
 * Pushes empty playing table to the playlist stack
 */
function pushPlaylist() {
    const table = getSongsTable(e => AppSingleton.get().playlistClick(e));
    table.classList.add('playlist-stack');

    const playing = playlists.querySelector('.playlist-stack');
    playlists.insertBefore(table, playing);
}

/**
 * Pops playlist from the playlist stack, sets the playing table as well
 */
function popPlaylist() {
    const tables = playlists.querySelectorAll('.playlist-stack');
    if (tables.length < 2) return;

    const playing = tables[0];
    playing.remove();
}

/**
 * Reorders playlists based on the given indexes.
 * @param {number[]} indexes - reorder indexes containing all playlists.
 */
function reorderPlaylists(indexes) {
    const wrapper = document.querySelector('.playlist-wrapper');
    const tables =
        Array.from(wrapper.querySelectorAll('#playlist .playlist-stack'));

    const reordered = indexes.map(i => tables[i]);
    reordered.forEach(table => wrapper.appendChild(table));
}

/**
 * Removes row of the playing playlist based on the given row ID
 * @param {number} id - row ID to be removed
 */
function removePlaylistRow(id) {
    const table = playlists.querySelector('#playlist .playlist-stack tbody');
    const rows = table.querySelectorAll('tr');
    if (rows.length <= id) return;

    if (rows[id].classList.contains('active') && rows[id].nextSibling !== null)
        rows[id].nextSibling.classList.add('active');
    rows[id].remove();
}

const playlistTabs = document.querySelector('#playlist .tabs .tabs-wrapper');
/**
 * Adds new playlist tab to the end
 */
function pushPlaylistTab(i = null) {
    const tab = document.createElement('button');
    tab.classList.add('tab');

    const id = i ?? playlistTabs.querySelectorAll('.tab').length;
    tab.textContent = `-${id}`;
    tab.onclick = () => AppSingleton.get().setPlaylistTab(id);

    playlistTabs.appendChild(tab);
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

const albumInfo = document.querySelector('#album-detail .info')
/**
 * Displays album in the album details page
 * @param {Album} album
 */
function displayAlbum(album) {
    albumInfo.querySelector('img').src =
        Album.getCover(album.artist, album.name);
    albumInfo.querySelector('.name').textContent = album.name;
    albumInfo.querySelector('.artist').textContent = album.artist;

    let other = album.getYear() !== '-' ? `${album.getYear()}  •  ` : '';
    albumInfo.querySelector('.other').textContent =
        `${other}${album.songs.length} songs`;

    const albumSongs = document.querySelector('#album-detail .songs');
    displaySongs(albumSongs, album.songs, false);
}

const artistInfo = document.querySelector('#artist-detail .info');
const artistAlbums = document.querySelector('#artist-detail .list');
/**
 * Displays artist in the artist details page
 * @param {Artist} artist
 */
function displayArtist(artist) {
    artistInfo.querySelector('.name').textContent = artist.name;
    artistInfo.querySelector('.other').textContent = artist.getOtherDetails();

    displayAlbums(artistAlbums, artist.albums);
    const artistSongs = document.querySelector('#artist-detail .songs');
    displaySongs(artistSongs, artist.songs);
}

/**
 * Displays songs in the given table
 * @param {HTMLTableElement} table - table element to display songs in
 * @param {Song[]} songs - songs to be displayed
 * @param {boolean} icons - whether to display icons
 */
function displaySongs(table, songs, icons = true) {
    const body = table.querySelector('tbody');
    body.innerHTML = '';

    songs.forEach((song, i) => {
        const row = song.getTableRow();
        row.dataset.index = i;

        if (!icons)
            row.querySelector('.cover').remove();

        body.appendChild(row);
    });
}

/**
 * Displays given albums in a given list
 * @param {HTMLDivElement} list
 * @param {Album[]} albums
 */
function displayAlbums(list, albums) {
    list.innerHTML = '';
    albums.forEach((album, i) => {
        const card = album.getCard();
        card.dataset.index = i;
        list.appendChild(card);
    });
}

function toggleBar() {
    bar.classList.toggle('expanded');
}

const tableTemplate = document.getElementById('songs-template');
/**
 * Gets empty songs table
 * @param {(e: MouseEvent)} onclick - on click event handler
 * @returns {HTMLTableElement} empty songs table
 */
function getSongsTable(onclick) {
    const cloned = tableTemplate.content.cloneNode(true);
    const table = cloned.querySelector('table');
    const tbody = table.querySelector('tbody');
    tbody.addEventListener('click', onclick);
    return table;
}

function spawnPlaylistTable() {
    const table = getSongsTable(e => AppSingleton.get().playlistClick(e));
    table.classList.add('playlist-stack', 'active');
    document.querySelector('#playlist .playlist-wrapper').appendChild(table);
}

function spawnAlbumDetailTable() {
    const table = getSongsTable(e => AppSingleton.get().albumSongClick(e));
    table.querySelector('.col-img').remove();
    table.querySelector('thead tr th').remove();
    document.getElementById('album-detail')
        .appendChild(table);
}

document.getElementById('library')
    .appendChild(getSongsTable(e => AppSingleton.get().libraryClick(e)));
document.getElementById('artist-detail')
    .appendChild(getSongsTable(e => AppSingleton.get().playlistClick(e)));
spawnPlaylistTable();
spawnAlbumDetailTable();
