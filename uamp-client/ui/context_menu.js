/**
 * @typedef {Object} ContextMenuItem
 * @property {string} label
 * @property {(object) => {}} action
 */

import Api from "../api.js";

/**
 * Abstract context menu implementation (menu on right click).
 */
class ContextMenu {
    constructor() {
        this.element = document.createElement("ul");
        this.element.classList.add("context-menu");
        document.body.appendChild(this.element);

        document.addEventListener("click", () => this.hide());
        document.addEventListener("scroll", () => this.hide(), true);
    }

    /**
     * Shows context menu with the given items.
     * @param {MouseEvent} event - the contextmenu event
     * @param {[ContextMenuItem]} items - items to be display in the menu
     * @param {any} data - data passed to the item `action`
     */
    show(event, items, data) {
        event.preventDefault();

        this.element.innerHTML = "";
        items.forEach((item) => {
            const li = document.createElement("li");
            li.textContent = item.label;
            li.onclick = () => {
                item.action(data);
                this.hide();
            };

            this.element.appendChild(li);
        });

        this.element.style.display = "block";
        let { pageX, pageY } = event;

        const mx = window.innerWidth - this.element.offsetWidth - 5;
        const my = window.innerHeight - this.element.offsetHeight - 5;

        this.element.style.left = `${Math.min(pageX, mx)}px`;
        this.element.style.top = `${Math.min(pageY, my)}px`;
    }

    /**
     * Hides the context menu.
     */
    hide() {
        this.element.style.display = "none";
    }
}

// ContextMenu singleton
export const contextMenu = new ContextMenu();

document.addEventListener("contextmenu", (e) => {
    const songRow = e.target.closest(".with-song-context tr");
    if (songRow) {
        contextMenu.show(e, SONG_CONTEXT_ITEMS, songRow.dataset.songId);
    }

    const albumCard = e.target.closest(".with-album-context .card");
    if (albumCard) {
        contextMenu.show(e, ALBUM_CONTEXT_ITEMS, albumCard.dataset.index);
    }

    const plRow = e.target.closest(".with-playlist-context tr");
    if (plRow) {
        const data = {
            id: Number(plRow.dataset.index),
            playlist: app.playlistTab,
        };
        contextMenu.show(e, PLAYLIST_CONTEXT_ITEMS, data);
    }
});

function insertSongs(songs, next, playlist = 0) {
    if (!songs || songs.length === 0) return;

    const id = next
        ? app.player.getPlaylist(playlist).getNextPId()
        : app.player.getPlaylist(playlist).songs.length;
    Api.insertIntoPlaylist(songs, id, playlist);
}

export const SONG_CONTEXT_ITEMS = [
    {
        label: "Play Next",
        action: (i) =>
            insertSongs([app.library.getSong(i)], true),
    },
    {
        label: "Add to Queue",
        action: (i) =>
            insertSongs([app.library.getSong(i)], false),
    },
];

export const ALBUM_CONTEXT_ITEMS = [
    {
        label: "Play Next",
        action: (id) => {
            const album = app.library.allAlbums[id];
            if (album) insertSongs(album.songs.get(), true);
        },
    },
    {
        label: "Add to Queue",
        action: (id) => {
            const album = app.library.allAlbums[id];
            if (album) insertSongs(album.songs.get(), false);
        },
    },
];

export const PLAYLIST_CONTEXT_ITEMS = [
    {
        label: "Play Next",
        action: ({ id, playlist }) => {
            const pl = app.player.getPlaylist(playlist);
            if (pl === null) return;
            Api.removeFromPlaylist([[id, id + 1]], playlist);
            insertSongs([pl.songs[id]], true, playlist);
        },
    },
    {
        label: "Remove from Playlist",
        action: ({ id, playlist }) =>
            Api.removeFromPlaylist([[id, id + 4]], playlist),
    },
];
