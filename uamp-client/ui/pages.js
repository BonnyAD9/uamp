import Album from "../library/album.js";

export function genericDisplayAlbums(albumsList, albums) {
    albumsList.innerHTML = "";
    albums.forEach((album) => {
        const card = album.getCard();
        card.dataset.index = album.id;
        albumsList.appendChild(card);
    });
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
export function displaySongs(table, songs, icons = true, id = null) {
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

/**
 * Gets the songs header table element for use in the screen header
 * @param {(string) => void} sortHandler - function handling the song sorting
 * @returns {HTMLTableElement} - table for displaying the songs header
 */
export function getSongsHeader(sortHandler = null) {
    const template = document.getElementById("songs-template");
    const cloned = template.content.cloneNode(true);

    const table = cloned.querySelector("table");
    table.classList.remove("songs");

    if (!sortHandler) return table;

    table.querySelectorAll("thead th span").forEach((label) => {
        label.addEventListener("click", () => sortHandler(label.dataset.sort));
    });
    return table;
}

/**
 * Gets custom header similar to the songs table header
 * @param {string[]} labels - labels to be in the header
 * @param {(string) => void} sortHandler - sort handler when label is clicked
 * @returns {HTMLTableElement} - table for displaying the custom header
 */
export function getCustomHeader(labels, sortHandler) {
    const table = document.createElement("table");
    const thead = document.createElement("thead");

    let row = document.createElement("tr");
    labels.forEach((label) => {
        const span = document.createElement("span");
        const key = label.trim().toLowerCase().replace(/\s+/g, "-");
        span.addEventListener("click", () => sortHandler(key));
        span.textContent = label;
        span.dataset.sort = key;

        const th = document.createElement("th");
        th.appendChild(span);
        row.appendChild(th);
    });

    thead.appendChild(row);
    table.appendChild(thead);
    return table;
}

function gradHoverListeners() {
    document.addEventListener("mousemove", (e) => {
        const target = e.target.closest(".grad-hover");
        if (!target) return;

        const rect = target.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;

        document.documentElement.style.setProperty("--mouse-x", `${x}px`);
        document.documentElement.style.setProperty("--mouse-y", `${y}px`);
    });
}

function card3DHover() {
    const MAX_ROTATION = 20;

    const lists = document.querySelectorAll(".list");
    lists.forEach((list) => {
        list.addEventListener("mousemove", (e) => {
            const target = e.target.closest(".card");
            if (!target) return;

            const rect = target.getBoundingClientRect();

            const x = e.clientX - rect.x - rect.width / 2;
            const y = e.clientY - rect.y - rect.height / 2;

            const xNorm = (x / (rect.width / 2)) * MAX_ROTATION;
            const yNorm = (y / (rect.height / 2)) * MAX_ROTATION;

            target.style.setProperty("--rot-x", `${-yNorm}deg`);
            target.style.setProperty("--rot-y", `${xNorm}deg`);

            const glow = target.querySelector(".glow");
            glow.style.setProperty("--x", `${-x * 2 + rect.width / 2}px`);
            glow.style.setProperty("--y", `${-y * 2 + rect.height / 2}px`);
        });
    });
}

export function spawnScreens() {
    // artistsScreen();
    // playlistScreen();

    gradHoverListeners();
    card3DHover();
}

document.querySelectorAll(".search-wrapper").forEach((wrapper) => {
    const btn = wrapper.querySelector("button");
    const input = wrapper.querySelector('input[type="search"]');
    btn.addEventListener("click", () => {
        input.value = "";
        input.dispatchEvent(new Event("input", { bubbles: true }));
    });
});
