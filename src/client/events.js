const AppSingleton = (() => {
    let instance = null;

    return {
        init(data) {
            instance = new App(data);
            return instance;
        },
        get() {
            return instance;
        }
    }
})();

const eventSource = new EventSource('/api/sub');

eventSource.addEventListener('set-all', e => {
    const app = AppSingleton.init(JSON.parse(e.data));
    app.displaySongs();
    app.displayPlaylist();
    app.updateAll();
});

eventSource.addEventListener('set-playlist', e => {
    const app = AppSingleton.get();
    setPlaylist(app, JSON.parse(e.data));
    playlistChanged(app);
});

eventSource.addEventListener('playback', e => {
    setPlayback(AppSingleton.get(), JSON.parse(e.data));
});

eventSource.addEventListener('playlist-jump', e => {
    const app = AppSingleton.get();
    const jump = JSON.parse(e.data);

    setPlayback(app, jump.playback);
    app.player.playlist.current = jump.position;
    setTimestamp(app, jump.timestamp);

    app.updateCurrent(app.getPlaying());
});

eventSource.addEventListener('seek', e => {
    const app = AppSingleton.get();
    app.position = JSON.parse(e.data);
});

eventSource.addEventListener('quitting', _ => console.log('Quitting...'));

eventSource.addEventListener('set-volume', e => {
    AppSingleton.get().updateVolume(JSON.parse(e.data));
});

eventSource.addEventListener('set-mute', e => {
    const app = AppSingleton.get();
    app.player.mute = JSON.parse(e.data);
    app.updateVolume(app.player.volume);
});

eventSource.addEventListener('pop-playlist', e => {
    const app = AppSingleton.get();
    const data = JSON.parse(e.data);

    popPlaylist(app);
    app.player.playlist.current = data.position;
    setTimestamp(app, data.timestamp);
    setPlayback(app, data.playback);

    playlistChanged(app);
});

eventSource.addEventListener('pop-set-playlist', e => {
    const app = AppSingleton.get();
    const data = JSON.parse(e.data);

    if (data.pop_cnt == 0)
        data.pop_cnt = app.player.playlist_stack.length;
    for (let i = 0; i < data.pop_cnt && popPlaylist(app); i++) { }

    setPlaylist(app, data.playlist);
    playlistChanged(app);
});

eventSource.addEventListener('set-playlist-add-policy', e => {
    const app = AppSingleton.get();
    app.player.playlist.add_policy = JSON.parse(e.data);
});

eventSource.addEventListener('set-playlist-end-action', e => {
    const app = AppSingleton.get();
    app.player.playlist.on_end = JSON.parse(e.data);
});

eventSource.addEventListener('push-playlist', e => {
    const app = AppSingleton.get();
    pushPlaylist(app, JSON.parse(e.data));
    playlistChanged(app);
});

eventSource.addEventListener('push-playlist-with-cur', e => {
    const app = AppSingleton.get();
    const data = JSON.parse(e.data);

    const current = app.player.playlist.current;
    if (current !== null) {
        app.player.playlist.songs.splice(current, 1);
    }

    pushPlaylist(app, data);
    app.displayPlaylist();
});

eventSource.addEventListener('queue', e => {
    const app = AppSingleton.get();
    app.player.playlist.songs.push(...JSON.parse(e.data));
    app.displayPlaylist();
});

eventSource.addEventListener('play-next', e => {
    const app = AppSingleton.get();

    const current = app.player.playlist.current;
    if (current === null) return;

    app.player.playlist.songs.splice(current + 1, 0, ...JSON.parse(e.data));
    app.displayPlaylist();
});

eventSource.addEventListener('restarting', e => console.log('Restaring...'));

eventSource.addEventListener('reorder-playlist-stack', e => { });

eventSource.addEventListener('play-tmp', e => { });

eventSource.addEventListener('new-server', _ => {
    console.log('You should use correct server!');
});

function setPlayback(app, playback) {
    app.player.state = playback;
    app.handleSongProgress();
    app.updatePlayBtn(app.isPlaying());
}

function setPlaylist(app, data) {
    setPlayback(app, data.playback);
    app.player.playlist = data.playlist;
    setTimestamp(app, data.timestamp);
}

function pushPlaylist(app, data) {
    app.player.playlist_stack.unshift(app.player.playlist);
    app.player.playlist = data.playlist;
    setTimestamp(app, data.timestamp);
    setPlayback(app, data.playback);
}

function popPlaylist(app) {
    if (app.player.playlist_stack.length == 0) return false;
    app.player.playlist = app.player.playlist_stack.shift();
    return true;
}

function playlistChanged(app) {
    app.displayPlaylist();
    app.updateSongs();
    app.updateCurrent(app.getPlaying());
}

function setTimestamp(app, timestamp) {
    app.position = timestamp;
    if (timestamp === null) return;

    app.getPlaying().length = timestamp.total;
    app.displayProgress(0);
}
