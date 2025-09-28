import Album from "../library/album.js";

const albumsList = document.querySelector("#albums .list");
/**
 * Displays given albums in a given list
 * @param {Album[]} albums
 */
export function displayAlbums(albums) {
    genericDisplayAlbums(albumsList, albums);
}

function genericDisplayAlbums(albumsList, albums) {
    albumsList.innerHTML = '';
    albums.forEach((album, i) => {
        const card = album.getCard();
        card.dataset.index = i;
        albumsList.appendChild(card);
    });
}

const albumInfo = document.querySelector("#album-detail .info");
// const albumBackdrop = document.querySelector('#album-detail .backdrop');
/**
 * Displays album in the album details page
 * @param {Album} album
 */
export function displayAlbum(album, id) {
    albumInfo.querySelector("img").src = Album.getCover(
        album.artist,
        album.name,
    );
    // albumBackdrop.src = Album.getCover(album.artist, album.name, 64);
    albumInfo.querySelector(".name").textContent = album.name;
    albumInfo.querySelector(".artist").textContent = album.artist;

    let other = album.getYear() !== "-" ? `${album.getYear()}  •  ` : "";
    albumInfo.querySelector(".other").textContent =
        `${other}${album.songs.get().length} songs`;

    displayAlbumSongs(album, id);
}

/**
 * Display album songs in the album details page
 * @param {Album} album
 * @param {number|null} id
 */
export function displayAlbumSongs(album, id) {
    const albumSongs = document.querySelector("#album-detail .songs");
    displaySongs(albumSongs, album.songs.get(), false, id);
}

const artistsList = document.querySelector("#artists .songs tbody");
/**
 * Displays given albums in a given list
 * @param {Album[]} artists
 */
export function displayArtists(artists) {
    artistsList.innerHTML = "";
    artists.forEach((artist, i) => {
        const row = artist.getTableRow();
        row.dataset.index = i;
        artistsList.appendChild(row);
    });
}

const artistInfo = document.querySelector("#artist-detail .info");
const artistAlbums = document.querySelector("#artist-detail .list");
/**
 * Displays artist in the artist details page
 * @param {Artist} artist
 */
export function displayArtist(artist, id) {
    artistInfo.querySelector(".name").textContent = artist.name;
    artistInfo.querySelector(".other").textContent = artist.getOtherDetails();

    genericDisplayAlbums(artistAlbums, artist.albums);
    displayArtistSongs(artist, id);
}

/**
 * Display artist songs in the artist details page
 * @param {Arist} artist
 * @param {number|null} id
 */
export function displayArtistSongs(artist, id) {
    const artistSongs = document.querySelector("#artist-detail .songs");
    displaySongs(artistSongs, artist.songs.get(), true, id);
}

/**
 * Displays songs in the given table
 * @param {HTMLTableElement} table - table element to display songs in
 * @param {Song[]} songs - songs to be displayed
 * @param {boolean} icons - whether to display icons
 */
function displaySongs(table, songs, icons = true, id = null) {
    const body = table.querySelector("tbody");
    body.innerHTML = "";

    songs.forEach((song, i) => {
        const row = song.getTableRow();
        row.dataset.index = i;
        row.dataset.songId = song.id;
        if (!icons) row.querySelector(".cover").remove();
        if (id === song.id) row.classList.add("active");

        body.appendChild(row);
    });
}
