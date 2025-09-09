import Duration from "./duration.js";

export default class Timestamp {
    /**
     * Creates new timestamp
     * @param {Duration} current 
     * @param {Duration} total 
     */
    constructor(current, total) {
        /** @type {Duration} */
        this.current = current;
        /** @type {Duration} */
        this.total = total;
    }

    /**
     * Creates new timestamp from the given object
     * @param {*} obj 
     * @returns 
     */
    static from(obj) {
        return new Timestamp(
            Duration.from(obj.current),
            Duration.from(obj.total)
        );
    }
}
