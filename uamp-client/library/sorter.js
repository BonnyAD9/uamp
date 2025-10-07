import Duration from "../helper/duration.js";

/**
 * Generic class that wraps given array and implements sorting for it.
 */
export default class Sorter {
    /**
     * Creates new generic sorter
     * @param {string} defaultKey - default sorting key
     * @param {any[]} arr - array to implement sorting for
     * @param {bool} ascending - true for ascending, false descending
     */
    constructor(defaultKey, arr = [], ascending = true) {
        this.arr = arr;
        this.key = defaultKey;
        this.defaultKey = defaultKey;
        this.ascending = ascending;

        this.sort();
    }

    /** @returns {any[]} - sorter array */
    get() {
        return this.arr;
    }

    /**
     * Sets the sorter array to given array and sorts it based on set sorting
     * @param {any[]} arr - array to be sorted
     */
    set(arr) {
        this.arr = arr;
        this.sort();
    }

    /**
     * Pushes the given item to the sorter array and sorts it
     * @param {any} item - item to be added
     */
    push(item) {
        this.arr.push(item);
        this.sort();
    }

    /** @returns {number} - number of items in the sorter array */
    len() {
        return this.arr.length;
    }

    /** Sorts the set array based on set sorting settings */
    sort() {
        this.arr = this.arr.sort((a, b) => {
            const res = Sorter.#cmp(a[this.key], b[this.key]);
            return this.ascending ? res : -res;
        });
    }

    /** Sets the sorting settings and sorts the array */
    sortBy(key, ascending) {
        this.key = key;
        this.ascending = ascending;
        this.sort();
    }

    /** Toggles the sorting settings and sorts the array */
    toggleSort(key) {
        if (this.key === key) {
            this.ascending = !this.ascending;
            if (this.ascending) {
                this.key = this.defaultKey;
            }
        } else {
            this.key = key;
            this.ascending = true;
        }
        this.sort();
    }

    /** Compares two values, returns pos value when A larger, neg B, 0 equal */
    static #cmp(valA, valB) {
        if (valA instanceof Duration) {
            return valA.cmp(valB);
        }

        if (Array.isArray(valA)) {
            valA = valA.length;
            valB = valB.length;
        }

        if (typeof valA === "string") {
            return valA.localeCompare(valB, undefined, {
                sensitivity: "accent",
            });
        }

        return valA - valB;
    }
}
