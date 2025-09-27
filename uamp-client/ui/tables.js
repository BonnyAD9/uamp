/** Highlights currently playing song in the library */
export const highlightLibrary = (id) =>
    highlightPlaying(id, document.querySelector("#library .songs tbody"));
/** Highlights currently playing song in both playlist and bar playlist */
export const highlightPlaylist = (id) => {
    highlightPlaying(
        id,
        document.querySelector("#playlist .playlist-stack tbody"),
    );
    highlightPlaying(id, document.querySelector(".bar .playlist .songs"));
};
/** Highlights currently playing song in the album songs */
export const highlightAlbumSong = (id) =>
    highlightPlaying(id, document.querySelector("#album-detail .songs tbody"));
/** Highlights currently playing song in the artist songs */
export const highlightArtistSong = (id) =>
    highlightPlaying(id, document.querySelector("#artist-detail .songs tbody"));

/**
 * Highlights song with given index in the given song elements
 * @param {?number} songId - index of the song to highlight
 * @param {HTMLElement} container - element containing the songs
 */
function highlightPlaying(songId, container) {
    for (const child of container.children) {
        const id = child.dataset.songId;
        child.classList.toggle("active", Number(id) === songId);
    }
}

const playlists = document.querySelector("#playlist .playlist-wrapper");
const playlistTabs = document.querySelector("#playlist .tabs .tabs-wrapper");
/** Pushes empty playing table to the playlist stack and adds new tab */
export function pushPlaylist() {
    const table = getTable(
        (e) => AppSingleton.get().playlistClick(e),
        ["playlist-stack"],
    );

    const playing = playlists.querySelector(".playlist-stack");
    playlists.insertBefore(table, playing);

    pushPlaylistTab();
}

/** Pushes new tab to the playlist tabs */
function pushPlaylistTab() {
    const tab = document.createElement("button");
    tab.classList.add("tab");

    const id = playlistTabs.querySelectorAll(".tab").length;
    tab.textContent = `-${id}`;
    tab.onclick = () => AppSingleton.get().setPlaylistTab(id);
    playlistTabs.appendChild(tab);
}

/** Pops playlist from the playlist stack and removes tab */
export function popPlaylist() {
    const tables = playlists.querySelectorAll(".playlist-stack");
    if (tables.length < 2) return;
    tables[0].remove();

    popPlaylistTab();
}

/** Pops tab from playlist tabs */
function popPlaylistTab() {
    const tabs = playlistTabs.querySelectorAll(".tab:not(#playingPlaylist)");
    if (tabs.length == 0) return;
    tabs[tabs.length - 1].remove();
}

/**
 * Reorders playlists based on the given indexes.
 * @param {number[]} indexes - reorder indexes containing all playlists.
 */
export function reorderPlaylists(indexes) {
    const wrapper = document.querySelector(".playlist-wrapper");
    const tables = Array.from(
        wrapper.querySelectorAll("#playlist .playlist-stack"),
    );

    const reordered = indexes.map((i) => tables[i]);
    reordered.forEach((table) => wrapper.appendChild(table));
}

/**
 * Removes row of the playing playlist based on the given row ID
 * @param {number} id - row ID to be removed
 */
export function removePlaylistRow(id) {
    const table = playlists.querySelector("#playlist .playlist-stack tbody");
    const rows = table.querySelectorAll("tr");
    if (rows.length <= id) return;

    if (rows[id].classList.contains("active") && rows[id].nextSibling !== null)
        rows[id].nextSibling.classList.add("active");
    rows[id].remove();
}

/**
 * Displays a playlist based on its ID in the playlist stack
 * @param {number} id - ID of the playlist stack
 */
export function showPlaylist(id) {
    const tabs = playlistTabs.querySelectorAll(".tab");
    const playlistStacks = playlists.querySelectorAll(".playlist-stack");
    for (let i = 0; i < tabs.length; i++) {
        tabs[i].classList.toggle("active", i === id);
        playlistStacks[i].classList.toggle("active", i === id);
    }
}

const tableTemplate = document.getElementById("songs-template");
/**
 * Gets empty songs table
 * @param {(e: MouseEvent)} onclick - on click event handler
 * @param {string[]} classes - classes to be added on the table
 * @returns {HTMLTableElement} empty songs table
 */
function getTable(onclick, classes = []) {
    const cloned = tableTemplate.content.cloneNode(true);
    const table = cloned.querySelector("table");
    table.classList.add(...classes);

    const tbody = table.querySelector("tbody");
    tbody.addEventListener("click", onclick);
    return table;
}

/**
 * Adds the table sort listeners on column labels
 * @param {HTMLTableElement} table - songs table
 * @param {(string) => void} sortHandler - function handling the song sorting
 */
function addTableSort(table, sortHandler) {
    const labels = table.querySelectorAll("thead th span");
    labels.forEach((label) => {
        label.addEventListener("click", () => sortHandler(label.dataset.sort));
    });
}

/**
 * Adds playlist tabs and tables to correspond the playlist stack
 * @param {number} n - stack length
 */
export function displayPlaylistStack(n) {
    playlistTabs
        .querySelectorAll(".tab:not(#playingPlaylist)")
        .forEach((tab) => tab.remove());
    playlists
        .querySelectorAll(".playlist-stack")
        .forEach((table, i) => i !== 0 && table.remove());

    for (let i = 1; i <= n; i++) {
        const cloned = getTable(
            (e) => AppSingleton.get().playlistClick(e),
            ["playlist-stack"],
        );
        playlists.appendChild(cloned);
        pushPlaylistTab();
    }
}

/** Spawns all the songs tables */
function spawnTables() {
    document
        .getElementById("library")
        .appendChild(getTable((e) => AppSingleton.get().libraryClick(e)));
    addTableSort(document.querySelector("#library .songs"), (key) =>
        AppSingleton.get().sortSongs(key),
    );

    document
        .getElementById("artist-detail")
        .appendChild(getTable((e) => AppSingleton.get().artistSongClick(e)));
    document
        .querySelector("#playlist .playlist-wrapper")
        .appendChild(
            getTable(
                (e) => AppSingleton.get().playlistClick(e),
                ["playlist-stack", "active"],
            ),
        );

    const table = getTable((e) => AppSingleton.get().albumSongClick(e));
    table.querySelector(".col-img").remove();
    table.querySelector("thead tr th").remove();
    document
        .querySelector("#album-detail .album-detail-wrapper")
        .appendChild(table);
}
spawnTables();
