export default class Api {
    /**
     * Sends API ctrl post request with the given data.
     * @param {any} data - data to be sent
     * @returns {Promise} fetch promise
     */
    static async postCtrl(data) {
        return fetch("/api/ctrl", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(data),
        });
    }

    /**
     * Sends push playlist API ctrl post request.
     * @param {Song[]} songs - songs to be pushed to the playlist
     * @param {number} pos - playing song in the playlist
     */
    static async pushPlaylist(songs, pos) {
        const data = {
            songs: songs.map((s) => s.id),
            position: Number(pos),
            play: true,
        };
        return Api.postCtrl({ SetPlaylist: data });
    }

    /**
     * Sends insert into playlist API ctrl post request.
     * @param {Song[]} songs - songs to be inserted
     * @param {number} pos - where to insert
     * @param {number} playlist - ID of playlist to insert into
     * @returns {Promise} fetch promise
     */
    static async insertIntoPlaylist(songs, pos, playlist = 0) {
        const data = {
            songs: songs.map((s) => s.id),
            position: Number(pos),
            playlist: Number(playlist),
        };
        return Api.postCtrl({ InsertIntoPlaylist: data });
    }

    /**
     * Sends remove from playlist API ctrl post request.
     * @param {number[][]} ranges - ranges of songs to remove
     * @param {number} playlist - ID of playlist to remove from
     * @returns {Promise} fetch promise
     */
    static async removeFromPlaylist(ranges, playlist = 0) {
        const data = {
            ranges,
            playlist: Number(playlist),
        };
        return Api.postCtrl({ RemoveFromPlaylist: data });
    }
}
