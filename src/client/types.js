const UNDEF_YEAR = 2147483647;

const songTemplate = document.getElementById('song-template');
const albumTemplate = document.getElementById('album-template');

class Song {
    /**
     * Creates new song
     * @param {string} path
     * @param {string} title
     * @param {string} artist 
     * @param {string} album 
     * @param {number} track 
     * @param {number} disc 
     * @param {number} year 
     * @param {Duration} length 
     * @param {string} genre 
     * @param {boolean} deleted 
     */
    constructor(
        path, title, artist, album, track, disc, year, length, genre,
        deleted = false
    ) {
        this.path = path;
        this.title = title;
        this.artist = artist;
        this.album = album;
        this.track = track;
        this.disc = disc;
        this.year = year;
        this.length = length;
        this.genre = genre;
        this.deleted = deleted;
    }

    static from(obj) {
        return new Song(
            obj.path,
            obj.title,
            obj.artist,
            obj.album,
            obj.track,
            obj.disc,
            obj.year,
            Duration.from(obj.length),
            obj.genre,
            obj.deleted
        );
    }

    /**
     * Gets songs release year, checkes for not set year
     * @returns {string} songs release year
     */
    getYear() {
        return this.year == UNDEF_YEAR ? '-' : `${this.year}`;
    }

    /**
     * Generates a table row with song details
     * @returns {HTMLTableRowElement} - generated song table row
     */
    getTableRow() {
        const cloned = songTemplate.content.cloneNode(true);
        const row = cloned.querySelector('tr');

        row.querySelector('.title').textContent = this.title;
        row.querySelector('.author').textContent = this.artist;
        row.querySelector('.album').textContent = this.album;
        row.querySelector('.year').textContent = this.getYear();
        row.querySelector('.length').textContent = this.length.format();
        row.querySelector('.genre').textContent = this.genre;
        row.querySelector('.track').textContent = this.track;
        row.querySelector('.disc').textContent = this.disc;

        return row;
    }

    /**
     * Gets uamp query for filtering the song
     * @returns uamp query string
     */
    getQuery() {
        const s = (text) => text.replaceAll('/', '//');
        return `n=/${s(this.title)}/.p=/${s(this.artist)}/.a=/` +
            `${s(this.album)}/.t=${this.track}.d=${this.disc}.y=${this.year}` +
            `.g=/${s(this.genre)}/`;
    }
}

class Album {
    /**
     * Creates new album
     * @param {string} name
     * @param {string} artist
     * @param {number} year 
     * @param {number[]} songs 
     */
    constructor(name, artist, year, songs = []) {
        this.name = name;
        this.artist = artist;
        this.year = year;
        this.songs = songs;
    }

    /**
     * Generates an album details card
     * @returns {HTMLElement} - generated album card
     */
    getCard() {
        const card = albumTemplate.content.cloneNode(true);
        card.querySelector('.name').textContent = this.name;
        card.querySelector('.artist').textContent = this.artist;
        return card;
    }
}

class Timestamp {
    /**
     * Creates new timestamp
     * @param {Duration} current 
     * @param {Duration} total 
     */
    constructor(current, total) {
        this.current = current;
        this.total = total;
    }

    static from(obj) {
        return new Timestamp(
            Duration.from(obj.current),
            Duration.from(obj.total)
        );
    }
}

class Duration {
    /**
     * Creates new duration
     * @param {number} secs
     * @param {number} nanos 
     */
    constructor(secs = 0, nanos = 0) {
        this.secs = secs;
        this.nanos = nanos;
        this.normalize();
    }

    static from(obj) {
        return new Duration(obj.secs, obj.nanos);
    }

    normalize() {
        if (this.nanos >= 1e9) {
            this.secs += Math.floor(this.nanos / 1e9);
            this.nanos %= 1e9;
        }
    }

    /**
     * Gets number of seconds in duration
     * @returns {number} number of seconds
     */
    toSecs() {
        return this.secs + this.nanos / 1e9;
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
