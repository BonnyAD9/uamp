/** Highlights currently playing song in the library */
export const highlightLibrary = (id) =>
    highlightPlaying(id, document.querySelector("#library .songs tbody"));
/** Highlights currently playing song in both playlist and bar playlist */
export const highlightPlaylist = (id) => {
    const playlist = document.querySelector("#playlist .playlist-stack tbody");
    highlightPlaying(id, playlist, "index");
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
function highlightPlaying(songId, container, attr = "songId") {
    for (const child of container.children) {
        const id = child.dataset[attr];
        child.classList.toggle("active", Number(id) === songId);
    }
}

/**
 * Displays the sorting in the thead of the table
 * @param {HTMLTableElement} table - table to which thead should display sort
 * @param {string} key - key of the sorting
 * @param {bool} direction - ascending when true, else descending
 */
export function displaySort(table, key, direction) {
    const attrs = table.querySelectorAll("thead th span");
    attrs.forEach((attr) => {
        attr.classList.remove("sorted", "asc", "desc");
        if (attr.dataset.sort === key) {
            attr.classList.add("sorted", direction ? "asc" : "desc");
        }
    });
}

const tableTemplate = document.getElementById("songs-template");
/**
 * Gets empty songs table
 * @param {(e: MouseEvent)} onclick - on click event handler
 * @param {string[]} classes - classes to be added on the table
 * @returns {HTMLTableElement} empty songs table
 */
export function getTable(onclick, sortHandler = null, classes = []) {
    const cloned = tableTemplate.content.cloneNode(true);
    const table = cloned.querySelector("table");
    table.classList.add(...classes);

    const tbody = table.querySelector("tbody");
    tbody.addEventListener("click", onclick);

    if (sortHandler != null) {
        addTableSort(table, sortHandler);
    }

    return table;
}

const tablehlTemplate = document.getElementById("songs-headerless-template");
export function getHeaderlessTable(onclick, classes = []) {
    const cloned = tablehlTemplate.content.cloneNode(true);
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
