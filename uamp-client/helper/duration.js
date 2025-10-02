export default class Duration {
    /**
     * Creates new duration
     * @param {number} secs
     * @param {number} nanos 
     */
    constructor(secs = 0, nanos = 0) {
        /** @type {number} */
        this.secs = secs;
        /** @type {number} */
        this.nanos = nanos;
        this.normalize();
    }

    /**
     * Creates new duration from the given object
     * @param {{ secs: number, nanos: number }} obj
     * @returns {Duration} created duration
     */
    static from(obj) {
        return new Duration(obj.secs, obj.nanos);
    }
    
    /**
     * Compares current duration with the given one
     * @param {Duration} other - duration to be compared to
     * @return {int} 1 if current is larger, else -1
     */
    cmp(other) {
        
    }

    /** Normalizes the Duration so the nanos don't exceed one second. */
    normalize() {
        if (this.nanos >= 1e9) {
            this.secs += Math.floor(this.nanos / 1e9);
            this.nanos %= 1e9;
        }
    }

    /**
     * Returns a new Duration representing the position within total duration
     * @param {number} percent - number between 0 and 1
     * @returns {Duration} new duration
     */
    fromPercent(percent) {
        percent = Math.max(0, Math.min(1, percent));

        const totalNanos = this.secs * 1e9 + this.nanos;
        const posNanos = Math.floor(totalNanos * percent);

        const secs = Math.floor(posNanos / 1e9);
        const nanos = posNanos % 1e9;
        return new Duration(secs, nanos);
    }

    /**
     * Gets number of seconds in duration
     * @returns {number} number of seconds
     */
    toSecs() {
        return this.secs + this.nanos / 1e9;
    }

    /**
     * Gets number of nanoseconds in durations
     * @returns {number} number of nanoseconds
     */
    toNanos() {
        return this.secs * 1e9 + this.nanos;
    }

    /**
     * Gets formated duration in format '%m:%ss'
     * @returns {string} formated duration
     */
    format() {
        const minutes = Math.floor(this.secs / 60);
        const seconds = this.secs % 60;
        if (minutes == 0 && seconds == 0) {
            return '-';
        }
        return `${minutes}:${seconds.toString().padStart(2, '0')}`;
    }
}
