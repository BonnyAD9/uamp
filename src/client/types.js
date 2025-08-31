const UNDEF_YEAR = 2147483647;

const songTemplate = document.getElementById('song-template');
const barSongTemplate = document.getElementById('bar-song-template');
const albumTemplate = document.getElementById('album-template');
const artistTemplate = document.getElementById('artist-template');

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
        /** @type {string} */
        this.path = path;
        /** @type {string} */
        this.title = title;
        /** @type {string} */
        this.artist = artist;
        /** @type {string} */
        this.album = album;
        /** @type {number} */
        this.track = track;
        /** @type {number} */
        this.disc = disc;
        /** @type {number} */
        this.year = year;
        /** @type {Duration} */
        this.length = length;
        /** @type {string} */
        this.genre = genre;
        /** @type {boolean} */
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
     * Gets songs release year, checks for not set year
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

        row.querySelector('img').src =
            Album.getCover(this.artist, this.album, 64);
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
     * Gets bar playlist song representation
     * @param {number} id 
     * @return {HTMLDivElement} bar playlist song
     */
    getBarRow(id) {
        const cloned = barSongTemplate.content.cloneNode(true);
        const item = cloned.querySelector('.item');

        item.querySelector('.id').textContent = id + 1;
        item.querySelector('.title').textContent = this.title;
        item.querySelector('.artist').textContent = this.artist;

        return item;
    }

    /**
     * Gets uamp query for filtering the song
     * @returns {string} uamp query string
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
     * @param {Song[]} songs 
     */
    constructor(name, artist, year, songs = []) {
        /** @type {string} */
        this.name = name;
        /** @type {string} */
        this.artist = artist;
        /** @type {number} */
        this.year = year;
        /** @type {Song[]} */
        this.songs = songs;
    }

    /**
     * Gets albums release year, checks for not set year
     * @returns {string} songs release year
     */
    getYear() {
        return this.year == UNDEF_YEAR ? '-' : `${this.year}`;
    }

    /**
     * Generates an album details card
     * @returns {HTMLElement} - generated album card
     */
    getCard() {
        const cloned = albumTemplate.content.cloneNode(true);
        const card = cloned.querySelector('.card');

        card.querySelector('img').src =
            Album.getCover(this.artist, this.name);
        card.querySelector('.name').textContent = this.name;
        card.querySelector('.artist').textContent = this.artist;
        return card;
    }

    /** 
     * Sorts albums songs by track number
     */
    sortByTrack() {
        this.songs.sort((a, b) => a.track - b.track);
    }

    /**
     * Gets uamp query for filtering the album
     * @returns {string} uamp query string
     */
    getQuery() {
        const s = (text) => text.replaceAll('/', '//');
        return `p=/${s(this.artist)}/.a=/${s(this.name)}/@/t`;
    }

    /**
     * Gets the API URL to get the album cover
     * @param {string} artist
     * @param {string} album
     * @return {string} API URL
     */
    static getCover(artist, album, size = null) {
        let req = `/api/img?artist=${encodeURIComponent(artist)}&album=` +
            `${encodeURIComponent(album)}&or=` +
            encodeURIComponent('/app/assets/svg/img_placeholder.svg');
        if (size !== null)
            req += `&size=${size}`;
        return req;
    }
}

class Artist {
    /**
     * Creates new artist
     * @param {string} name
     * @param {Song[]} songs
     * @param {Album[]} albums
     */
    constructor(name, songs = [], albums = []) {
        /** @type {string} */
        this.name = name;
        /** @type {Song[]} */
        this.songs = songs;
        /** @type {Album[]} */
        this.albums = albums;
    }

    /**
     * Gets albums and songs count string
     * @returns {string} the details string
     */
    getOtherDetails() {
        return `${this.albums.length} albums  •  ${this.songs.length} songs`;
    }

    /**
     * Generates a table row with artist details
     * @returns {HTMLTableRowElement} - generated artist table row
     */
    getTableRow() {
        const cloned = artistTemplate.content.cloneNode(true);
        const row = cloned.querySelector('tr');

        row.querySelector('.artist').textContent = this.name;
        row.querySelector('.other').textContent = this.getOtherDetails();

        const albums = row.querySelector('.albums-preview');
        this.albums.forEach((album, i) => {
            const img = document.createElement('img');
            img.src = Album.getCover(album.artist, album.name, 64);
            img.title = album.name;
            img.dataset.index = i;
            albums.appendChild(img);
        });

        return row;
    }

    /**
     * Gets uamp query for filtering the artist
     * @returns {string} uamp query string
     */
    getQuery() {
        const s = (text) => text.replaceAll('/', '//');
        return `p=/${s(this.name)}/`;
    }

    /**
     * Sorts artists albums by release year
     */
    sortAlbums() {
        this.albums.sort((a, b) => a.year - b.year);
    }
}

class Timestamp {
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
        /** @type {number} */
        this.secs = secs;
        /** @type {number} */
        this.nanos = nanos;
        this.normalize();
    }

    static from(obj) {
        return new Duration(obj.secs, obj.nanos);
    }

    /**
     * Normalizes the Duration so the nanos don't exceed one second.
     */
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
