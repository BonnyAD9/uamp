import App from "./app.js";
import "./colors.js";
import { spawnScreens } from "./ui/pages.js";
import { removePlaylistRow } from "./ui/tables.js";

spawnScreens();

const AppSingleton = (() => {
    let instance = null;

    return {
        async init(data) {
            instance = await App.init(data);
            return instance;
        },
        get() {
            return instance;
        },
    };
})();
window.AppSingleton = AppSingleton;

const eventSource = new EventSource("/api/sub");

eventSource.addEventListener("set-all", async (e) => {
    const app = await AppSingleton.init(JSON.parse(e.data));
    app.displaySongs();

    setTimeout(() => {
        app.updateAll();
        app.displayAlbums();
        app.displayArtists();
    }, 0);
});

eventSource.addEventListener("set-playlist", (e) => {
    setPlaylistEvent(AppSingleton.get(), JSON.parse(e.data));
});

eventSource.addEventListener("playback", (e) => {
    AppSingleton.get().setPlayback(JSON.parse(e.data));
});

eventSource.addEventListener("playlist-jump", (e) => {
    playlistJumpEvent(AppSingleton.get(), JSON.parse(e.data));
});

eventSource.addEventListener("seek", (e) => {
    AppSingleton.get().setTimestamp(JSON.parse(e.data));
});

eventSource.addEventListener("quitting", (_) => console.log("Quitting..."));

eventSource.addEventListener("set-volume", (e) => {
    AppSingleton.get().player.setVolume(JSON.parse(e.data));
});

eventSource.addEventListener("set-mute", (e) => {
    AppSingleton.get().player.setMute(JSON.parse(e.data));
});

eventSource.addEventListener("pop-playlist", (e) => {
    const app = AppSingleton.get();
    const data = JSON.parse(e.data);

    app.popPlaylist(data.pop_cnt);
    playlistJumpEvent(app, data.playlist);
});

eventSource.addEventListener("pop-set-playlist", (e) => {
    const app = AppSingleton.get();
    const data = JSON.parse(e.data);

    app.popPlaylist(data.pop_cnt);
    setPlaylistEvent(app, data.playlist);
});

eventSource.addEventListener("set-playlist-add-policy", (e) => {
    const app = AppSingleton.get();
    app.player.playlist.add_policy = JSON.parse(e.data);
});

eventSource.addEventListener("set-playlist-end-action", (e) => {
    const app = AppSingleton.get();
    app.player.playlist.on_end = JSON.parse(e.data);
});

eventSource.addEventListener("push-playlist", (e) => {
    pushPlaylistEvent(AppSingleton.get(), JSON.parse(e.data));
});

eventSource.addEventListener("push-playlist-with-cur", (e) => {
    const app = AppSingleton.get();
    const current = app.player.playlist.current;
    if (current !== null) {
        app.player.playlist.songs.splice(current, 1);
        removePlaylistRow(current);
    }

    pushPlaylistEvent(app, JSON.parse(e.data));
});

eventSource.addEventListener("queue", (e) => {
    const app = AppSingleton.get();
    app.player.playlist.songs.push(...JSON.parse(e.data));
    if (app.playlistTab === 0) {
        app.displayPlaylist();
        app.createBarSongs();
    }
});

eventSource.addEventListener("play-next", (e) => {
    const app = AppSingleton.get();

    const current = app.player.playlist.current;
    if (current === null) return;

    app.player.playlist.songs.splice(current + 1, 0, ...JSON.parse(e.data));
    if (app.playlistTab === 0) {
        app.displayPlaylist();
        app.createBarSongs();
    }
});

eventSource.addEventListener("restarting", (_) => console.log("Restarting..."));

eventSource.addEventListener("reorder-playlist-stack", (e) => {
    const app = AppSingleton.get();
    const data = JSON.parse(e.data);

    const order = extendArray(data.order, app.player.playlist_stack.length);
    app.reorderPlaylists(order);
    playlistJumpEvent(app, data.position);
});

eventSource.addEventListener("play-tmp", (e) => {
    const app = AppSingleton.get();
    const data = JSON.parse(e.data);

    app.pushTmpSongs(data.songs);
    app.setPlaylist(data.playlist);
    app.setPlayback(data.playback);
    app.setTimestamp(data.timestamp);
});

eventSource.addEventListener("new-server", (e) => {
    const { address, port } = JSON.parse(e.data);
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
});

eventSource.addEventListener("client-changed", (_) => window.location.reload());

eventSource.addEventListener("config-changed", (e) => {
    const app = AppSingleton.get();
    if (app === null) return;
    app.config = JSON.parse(e.data);
});

function playlistJumpEvent(app, data) {
    app.player.setCurrent(data.position);
    app.setPlayback(data.playback);
    app.setTimestamp(data.timestamp);
}

function setPlaylistEvent(app, data) {
    app.setPlaylist(data.playlist);
    app.setTimestamp(data.timestamp);
    app.setPlayback(data.playback);
}

function pushPlaylistEvent(app, data) {
    app.pushPlaylist(data.playlist);
    app.setTimestamp(data.timestamp);
    app.setPlayback(data.playback);
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
