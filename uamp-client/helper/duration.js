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
        if (obj == null) {
            return null;
        }
        return new Duration(obj.secs, obj.nanos);
    }

    /** Normalizes the Duration so the nanos don't exceed one second. */
    normalize() {
        if (this.nanos >= 1e9) {
            this.secs += Math.floor(this.nanos / 1e9);
            this.nanos %= 1e9;
        }
    }

    add(other) {
        if (!other) return this;

        this.secs += other.secs;
        this.nanos += other.nanos;
        this.normalize();
        return this;
    }

    /**
     * Compares two durations
     * @param {Duration} other - duration to be compared to current
     * @returns {number} pos. number if current greater, neg. if other, 0 equal
     */
    cmp(other) {
        const diff = this.secs - other.secs;
        if (diff != 0) return diff;
        return this.nanos - other.nanos;
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
        const days = Math.floor(this.secs / 86400);
        const hours = Math.floor((this.secs % 86400) / 3600);
        const minutes = Math.floor((this.secs % 3600) / 60);
        const seconds = this.secs % 60;

        const pad = (num) => num.toString().padStart(2, "0");

        if (days > 0) {
            return `${days}d:${pad(hours)}:${pad(minutes)}:${pad(seconds)}`;
        }
        if (hours > 0) {
            return `${hours}:${pad(minutes)}:${pad(seconds)}`;
        }
        return `${minutes}:${pad(seconds)}`;
    }
}
