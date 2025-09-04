export default class Songs {
    /**
     * Creates new songs
     * @param {Song[]} songs - list of songs
     * @param {string} order - song attribute to sort the songs by
     */
    constructor(songs = [], sort = "id", ascending = true) {
        /** @type {Song[]} */
        this.songs = songs;
        /** @type {string} */
        this.sort = sort;
        /** @type {boolean} */
        this.ascending = ascending;

        this.#sortSongs();
    }

    /**
     * Sorts the songs by given key in direction based on ascending
     * @param {string} key - sorting key
     * @param {boolean} ascending - whether should sort in ascending form
     */
    sortBy(key, ascending) {
        this.sort = key;
        this.ascending = ascending;
        this.#sortSongs();
    }

    /**
     * Toggles sorting - if key for the first time -> ascending, 
     * second -> descending, third -> back to original order.
     * @param {string} key - key to toggle sort by
     */
    toggleSort(key) {
        if (this.sort === key) {
            this.ascending = !this.ascending;
            if (this.ascending) {
                this.key === 'id';
            }
        } else {
            this.key = key;
            this.ascending = true;
        }
        this.#sortSongs();
    }

    /** Sorts the songs by set key and based on set order */
    #sortSongs() {
        this.songs = this.songs.sort((a, b) => {
            let valA = a[this.sort];
            let valB = b[this.sort];
            if (typeof valA === 'Duration') {
                valA = valA.toNanos();
                valB = valB.toNanos();
            }

            let result;
            if (typeof valA === 'string') {
                result = valA.localeCompare(
                    valB, undefined, { sensitivity: 'accent' }
                );
            } else {
                result = valA - valB;
            }

            return this.ascending ? result : -result;
        });
    }
}
