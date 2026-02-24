import App from "./app.js";
import "./ui/components.js";
import "./ui/context-menu.js";
import { spawnScreens } from "./ui/pages.js";

spawnScreens();

window.app = new App();
window.addEventListener("load", connectSSE);

/** Object containing implemented SSE event handlings. */
const SSE_HANDLERS = {
    "set-all": async (data) => {
        data.sse.forEach((event) => {
            if (!SSE_HANDLERS[event]) {
                eventSource.addEventListener(event, (e) => connectSSE());
            }
        });

        await app.init(data);
        setTimeout(() => {
            app.updateAll();
            app.displayAlbums();
            app.displayArtists();
        }, 0);
    },
    "set-playlist": setPlaylistEvent,
    playback: (data) => app.setPlayback(data),
    "playlist-jump": playlistJumpEvent,
    seek: (data) => app.setTimestamp(data),
    quitting: () => console.log("Quitting..."),
    "set-volume": (data) => app.player.setVolume(data),
    "set-mute": (data) => app.player.setMute(data),
    "pop-playlist": ({ pop_cnt, playlist }) => {
        app.popPlaylist(pop_cnt);
        playlistJumpEvent(playlist);
    },
    "pop-set-playlist": ({ pop_cnt, playlist }) => {
        app.popPlaylist(pop_cnt);
        setPlaylistEvent(playlist);
    },
    "set-playlist-add-policy": (data) =>
        (app.player.playlist.add_policy = data),
    "set-playlist-end-action": (data) => (app.player.playlist.on_end = data),
    "push-playlist": pushPlaylistEvent,
    "push-playlist-with-cur": (data) => {
        const current = app.player.playlist.current;
        if (current !== null) {
            app.player.playlist.songs.splice(current, 1);
            app.playlistScreen.removeRow(current);
        }

        pushPlaylistEvent(data);
    },
    "insert-into-playlist": (data) => {
        playlistAction(data, ({ songs, position }, playlist) => {
            const newSongs = songs.map((id) => app.library.getSong(id));
            playlist.songs.splice(position, 0, ...newSongs);
            if (playlist.current !== null && position <= playlist.current) {
                playlist.current += songs.length;
            }
        });
    },
    "remove-from-playlist": (data) => {
        playlistAction(data, (data, playlist) => {
            data.ranges.sort((a, b) => b[0] - a[0]);
            data.ranges.forEach((range) => {
                const [start, end] = range;
                const deleted = end - start;
                playlist.songs.splice(start, deleted);

                if (playlist.current !== null && playlist.current >= end) {
                    playlist.current -= deleted;
                }
            });
        });
    },
    restarting: () => console.log("Restarting..."),
    "reorder-playlist-stack": ({ order, position }) => {
        const ord = extendArray(order, app.player.playlist_stack.length);
        app.reorderPlaylists(ord);
        playlistJumpEvent(position);
    },
    "play-tmp": (data) => {
        app.pushTmpSongs(data.songs);
        app.setPlaylist(data.playlist);
        app.setPlayback(data.playback);
        app.setTimestamp(data.timestamp);
    },
    "new-server": ({ address, port }) => {
        const server = `http://${address}:${port}`;
        const ping = `${server}/api/marco`;

        const interval = setInterval(async () => {
            fetch(ping, { cache: "no-store" })
                .then((res) => res.text())
                .then((text) => {
                    if (text === "polo") {
                        clearInterval(interval);
                        window.location.href = `${server}/app`;
                    }
                })
                .catch((_) => {});
        }, 1000);
    },
    "client-changed": () => window.location.reload(),
    "config-changed": async (data) => app.config.init(data),
    "remove-playlist": (id) => app.removePlaylist(id),
};

/** @type {EventSource|null} */
let eventSource;
/** Connects SSE to the server with auto-reconnecting. */
function connectSSE() {
    if (eventSource) eventSource.close();

    eventSource = new EventSource("/api/sub");
    Object.entries(SSE_HANDLERS).forEach(([event, handler]) => {
        eventSource.addEventListener(event, async (e) => {
            const data = e.data ? JSON.parse(e.data) : null;
            await handler(data);
        });
    });

    eventSource.onopen = () => console.log("SSE connected.");
    eventSource.onerror = () => {
        console.error(
            "Error happened in SSE connection. Reconnecting in 3s...",
        );
        eventSource.close();
        setTimeout(connectSSE, 3000);
    };
}

function playlistJumpEvent(data) {
    app.setCurrent(data.position);
    app.setPlayback(data.playback);
    app.setTimestamp(data.timestamp);
}

function setPlaylistEvent(data) {
    app.setPlaylist(data.playlist);
    app.setTimestamp(data.timestamp);
    app.setPlayback(data.playback);
}

function pushPlaylistEvent(data) {
    app.pushPlaylist(data.playlist);
    app.setTimestamp(data.timestamp);
    app.setPlayback(data.playback);
}

/**
 * Appplies given action on a playlist
 * @param {object} data - event data
 * @param {(object, Playlist) => void} action - action performed on playlist
 */
function playlistAction(data, action) {
    const playlist = app.player.getPlaylist(data.playlist);
    if (playlist === null) return;

    action(data, playlist);
    if (app.playlistTab === data.playlist) {
        app.displayPlaylist();
        app.createBarSongs();
    }
}

function extendArray(arr, max) {
    const set = new Set(arr);
    const missing = [];
    for (let i = 0; i <= max; i++) {
        if (!set.has(i)) {
            missing.push(i);
        }
    }

    return arr.concat(missing);
}
