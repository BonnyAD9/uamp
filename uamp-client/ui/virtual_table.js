import Song from "../library/song.js";

export default class VirtualTable {
    /**
     * Creates new virtual table
     * @param {() => Song[]} getSongs - gets songs to be displayed
     * @param {string} container - scrollable container selector
     * @param {string} table - table selector located inside the container
     * @param {() => number} getCurrentId - gets song ID to be highlighted
     * @param {boolean} isTable - normal table when true, list when false
     * @param {boolean} autoScroll - automatically scrolls to current song
     */
    constructor(
        getSongs,
        container,
        table,
        getCurrentId,
        isTable = true,
        autoScroll = false,
    ) {
        /** @type {Song[]} */
        this.getSongs = getSongs;
        /** @type {HTMLElement} */
        this.container = document.querySelector(container);
        /** @type {string} */
        this.tableSelector = table;

        /** @type {number} */
        this.rowHeight = isTable ? 42 : 32;
        /** @type {(Song, number) => HTMLElement} */
        this.getSongElement = isTable
            ? (song, _) => song.getTableRow()
            : (song, id) => song.getBarRow(id);
        /** @type {() => number} */
        this.getCurrentId = getCurrentId;

        /** @type {number} */
        this.start = 0;
        /** @type {number} */
        this.end = 0;

        this.autoScroll = false;
        this.autoScrollEnabled = autoScroll;
        this.ignoreScroll = false;
        this.container.addEventListener("scroll", () => this.update());
    }

    /** Displays songs in the given containers table using virtual scrolling. */
    render() {
        this.autoScroll = this.autoScrollEnabled;
        const current = this.getCurrentId();
        const songs = this.getSongs();

        const table = this.container.querySelector(this.tableSelector);
        table.innerHTML = "";

        const top = document.createElement("tr");
        top.classList.add("spacer", "spacer-top");
        table.appendChild(top);

        const bottom = document.createElement("tr");
        bottom.classList.add("spacer", "spacer-bottom");
        table.appendChild(bottom);

        let start, end;
        if (this.autoScroll) {
            ({ start, end } = this.#getCurrentPos(songs, current, top, bottom));
        } else {
            ({ start, end } = this.#getBufferPos(songs.length, top, bottom));
        }

        const fragment = document.createDocumentFragment();
        for (let i = start; i < end; i++)
            fragment.appendChild(this.#getRow(songs, i, current));
        top.after(fragment);

        this.start = start;
        this.end = end;

        if (this.autoScroll) {
            this.ignoreScroll = true;
            this.container.scrollTop = (start + 2) * this.rowHeight;
            requestAnimationFrame(() => (this.ignoreScroll = false));
        }
    }

    /** Updates songs table in the given container with virtual scrolling. */
    update() {
        if (this.ignoreScroll) return;

        this.autoScroll = false;
        const current = this.getCurrentId();
        const songs = this.getSongs();

        const table = this.container.querySelector(this.tableSelector);
        const top = table.querySelector(".spacer-top");
        const bottom = table.querySelector(".spacer-bottom");
        const { start, end } = this.#getBufferPos(songs.length, top, bottom);

        for (let i = this.start - 1; i >= start; i--)
            top.after(this.#getRow(songs, i, current));
        for (let i = this.end; i < end; i++) {
            bottom.before(this.#getRow(songs, i, current));
        }

        const removeRow = (row) => {
            if (row && !row.classList.contains("spacer"))
                table.removeChild(row);
        };
        for (let i = this.start; i < start; i++) removeRow(top.nextSibling);
        for (let i = this.end; i > end; i--) removeRow(bottom.previousSibling);

        this.start = start;
        this.end = end;
    }

    /**
     * Gets buffer position for the virtual scrolling and updates spacers
     * @param {number} songCnt - total number of songs in the virtual table
     * @param {HTMLElement} topSpacer - top spacer row
     * @param {HTMLElement} bottomSpacer - bottom spacer row
     * @returns {{ start: number, end: number }} buffer boundaries
     */
    #getBufferPos(songCnt, topSpacer, bottomSpacer) {
        const viewHeight = this.container.clientHeight;

        const visibleCnt = Math.ceil(viewHeight / this.rowHeight) + 1;
        const scrollTop = this.container.scrollTop;
        const start = Math.max(0, Math.floor(scrollTop / this.rowHeight) - 2);
        const end = Math.min(songCnt, start + visibleCnt);

        topSpacer.style.height = `${start * this.rowHeight}px`;
        bottomSpacer.style.height = `${(songCnt - end) * this.rowHeight}px`;
        return { start, end };
    }

    #getCurrentPos(songs, current, topSpacer, bottomSpacer) {
        const viewHeight = this.container.clientHeight;
        const songCnt = songs.length;

        const visible = Math.ceil(viewHeight / this.rowHeight);
        const currentPos = songs.findIndex((s) => s.id == current);

        const top = Math.ceil(currentPos - visible * 0.5);
        const start = Math.max(0, Math.min(top, songCnt - visible));
        const end = Math.min(songCnt, start + visible);

        topSpacer.style.height = `${start * this.rowHeight}px`;
        bottomSpacer.style.height = `${(songCnt - end) * this.rowHeight}px`;

        console.log(start, currentPos, end);
        return { start, end };
    }

    /**
     * Gets table row for the given song id
     * @param {Song[]} songs - list of songs
     * @param {number} id - song id to get the row for
     * @param {number} current - song id of the currently playing song
     * @returns {HTMLElement} table row for the given song id
     */
    #getRow(songs, id, current) {
        const song = songs[id];
        const row = this.getSongElement(song, id);
        row.dataset.index = id;
        row.dataset.songId = song.id;
        if (song.id === current) row.classList.add("active");
        return row;
    }
}
