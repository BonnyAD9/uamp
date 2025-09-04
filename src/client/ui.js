import Album from "./library/album.js";

const pausePlayP1 = document.getElementById("from_pause_to_play_p1");
const playPauseP1 = document.getElementById("from_play_to_pause_p1");
const pausePlayP2 = document.getElementById("from_pause_to_play_p2");
const playPauseP2 = document.getElementById("from_play_to_pause_p2");
/**
 * Updates the play button based on the playing state
 * @param {boolean} playing - whether player is playing or not
 */
export function updatePlayBtn(playing) {
    if (playing) {
        playPauseP1.beginElement();
        playPauseP2.beginElement();
    } else {
        pausePlayP1.beginElement();
        pausePlayP2.beginElement();
    }
}

export const songIcon = document.querySelector('.bar .info .info-pic img');
const songTitle = document.querySelector('.bar .info .title h3');
const songArtist = document.querySelector('.bar .info .title h4');
const barBackdrop = document.querySelector('.bar .backdrop');
/**
 * Updates the currently playing song info
 * @param {?Song} song - currently playing song
 */
export function updateCurrent(song) {
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
export function updateVolume(volume, mute = false) {
    const perVolume = Math.round(volume * 100);
    volumeSlider.value = perVolume;
    volumeValue.textContent = perVolume;

    const level = volume === 1.0 ? 4 : Math.ceil(volume * 3);
    let icon = `${mute ? 'no_' : ''}volume_${level}.svg`;
    volumeIcon.src = `assets/svg/${icon}`;
}

export const highlightLibrary = id =>
    highlightPlaying(id, document.querySelector('#library .songs tbody'));
export const highlightPlaylist = id => {
    highlightPlaying(
        id, document.querySelector('#playlist .playlist-stack tbody')
    );
    highlightPlaying(id, document.querySelector('.bar .playlist .songs'));
}
export const highlightAlbumSong = id =>
    highlightPlaying(id, document.querySelector('#album-detail .songs tbody'));
export const highlightArtistSong = id =>
    highlightPlaying(id, document.querySelector('#artist-detail .songs tbody'));

/**
 * Highlights song with given index in the given song elements
 * @param {?number} songId - index of the song to highlight
 * @param {HTMLElement} container - element containing the songs
 */
export function highlightPlaying(songId, container) {
    for (const child of container.children) {
        const id = child.dataset.songId;
        child.classList.toggle('active', Number(id) === songId);
    }
}

export const playlists = document.querySelector('#playlist .playlist-wrapper');
/**
 * Pushes empty playing table to the playlist stack
 */
export function pushPlaylist() {
    const table = getSongsTable(e => AppSingleton.get().playlistClick(e));
    table.classList.add('playlist-stack');

    const playing = playlists.querySelector('.playlist-stack');
    playlists.insertBefore(table, playing);
}

/**
 * Pops playlist from the playlist stack, sets the playing table as well
 */
export function popPlaylist() {
    const tables = playlists.querySelectorAll('.playlist-stack');
    if (tables.length < 2) return;

    const playing = tables[0];
    playing.remove();
}

/**
 * Reorders playlists based on the given indexes.
 * @param {number[]} indexes - reorder indexes containing all playlists.
 */
export function reorderPlaylists(indexes) {
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
export function removePlaylistRow(id) {
    const table = playlists.querySelector('#playlist .playlist-stack tbody');
    const rows = table.querySelectorAll('tr');
    if (rows.length <= id) return;

    if (rows[id].classList.contains('active') && rows[id].nextSibling !== null)
        rows[id].nextSibling.classList.add('active');
    rows[id].remove();
}

export const playlistTabs =
    document.querySelector('#playlist .tabs .tabs-wrapper');
/**
 * Adds new playlist tab to the end
 */
export function pushPlaylistTab(i = null) {
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
export function popPlaylistTab() {
    const tabs = playlistTabs.querySelectorAll('.tab:not(#playingPlaylist)');
    if (tabs.length == 0) return;
    tabs[tabs.length - 1].remove();
}

/**
 * Displays a playlist based on its ID in the playlist stack
 * @param {number} id - ID of the playlist stack
 */
export function showPlaylist(id) {
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

const albumInfo = document.querySelector('#album-detail .info');
const albumBackdrop = document.querySelector('#album-detail .backdrop');
/**
 * Displays album in the album details page
 * @param {Album} album
 */
export function displayAlbum(album, id) {
    albumInfo.querySelector('img').src =
        Album.getCover(album.artist, album.name);
    // albumBackdrop.src = Album.getCover(album.artist, album.name, 64);
    albumInfo.querySelector('.name').textContent = album.name;
    albumInfo.querySelector('.artist').textContent = album.artist;

    let other = album.getYear() !== '-' ? `${album.getYear()}  •  ` : '';
    albumInfo.querySelector('.other').textContent =
        `${other}${album.songs.length} songs`;

    const albumSongs = document.querySelector('#album-detail .songs');
    displaySongs(albumSongs, album.songs, false, id);
}

const artistInfo = document.querySelector('#artist-detail .info');
const artistAlbums = document.querySelector('#artist-detail .list');
/**
 * Displays artist in the artist details page
 * @param {Artist} artist
 */
export function displayArtist(artist, id) {
    artistInfo.querySelector('.name').textContent = artist.name;
    artistInfo.querySelector('.other').textContent = artist.getOtherDetails();

    displayAlbums(artistAlbums, artist.albums);
    const artistSongs = document.querySelector('#artist-detail .songs');
    displaySongs(artistSongs, artist.songs, true, id);
}

/**
 * Displays songs in the given table
 * @param {HTMLTableElement} table - table element to display songs in
 * @param {Song[]} songs - songs to be displayed
 * @param {boolean} icons - whether to display icons
 */
export function displaySongs(table, songs, icons = true, id = null) {
    const body = table.querySelector('tbody');
    body.innerHTML = '';

    songs.forEach((song, i) => {
        const row = song.getTableRow();
        row.dataset.index = i;
        row.dataset.songId = song.id;
        if (!icons)
            row.querySelector('.cover').remove();
        if (id === song.id)
            row.classList.add('active')

        body.appendChild(row);
    });
}

/**
 * Displays given albums in a given list
 * @param {HTMLDivElement} list
 * @param {Album[]} albums
 */
export function displayAlbums(list, albums) {
    list.innerHTML = '';
    albums.forEach((album, i) => {
        const card = album.getCard();
        card.dataset.index = i;
        list.appendChild(card);
    });
}

const bar = document.querySelector('section.bar');
export function toggleBar() {
    bar.classList.toggle('expanded');
    if (bar.classList.contains('expanded'))
        setTimeout(() => AppSingleton.get().createBarSongs(), 200);
}
window.toggleBar = toggleBar;

export const tableTemplate = document.getElementById('songs-template');
/**
 * Gets empty songs table
 * @param {(e: MouseEvent)} onclick - on click event handler
 * @returns {HTMLTableElement} empty songs table
 */
export function getSongsTable(onclick) {
    const cloned = tableTemplate.content.cloneNode(true);
    const table = cloned.querySelector('table');
    const tbody = table.querySelector('tbody');
    tbody.addEventListener('click', onclick);
    return table;
}

export function spawnPlaylistTable() {
    const table = getSongsTable(e => AppSingleton.get().playlistClick(e));
    table.classList.add('playlist-stack', 'active');
    document.querySelector('#playlist .playlist-wrapper').appendChild(table);
}

export function spawnAlbumDetailTable() {
    const table = getSongsTable(e => AppSingleton.get().albumSongClick(e));
    table.querySelector('.col-img').remove();
    table.querySelector('thead tr th').remove();
    document.querySelector('#album-detail .album-detail-wrapper')
        .appendChild(table);
}

document.getElementById('library')
    .appendChild(getSongsTable(e => AppSingleton.get().libraryClick(e)));
document.getElementById('artist-detail')
    .appendChild(getSongsTable(e => AppSingleton.get().artistSongClick(e)));
spawnPlaylistTable();
spawnAlbumDetailTable();
