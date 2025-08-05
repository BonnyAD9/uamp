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
    const data = JSON.parse(e.data);

    setPlayback(app, data.playback);
    app.player.playlist = data.playlist;
    app.position = data.timestamp;
    
    app.updatePlaylist();
});

eventSource.addEventListener('playback', e => {
    setPlayback(AppSingleton.get(), JSON.parse(e.data));
});

eventSource.addEventListener('playlist-jump', e => {
    const app = AppSingleton.get();
    const jump = JSON.parse(e.data);

    setPlayback(app, jump.playback);
    app.player.playlist.current = jump.position;
    app.position = jump.timestamp;

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

eventSource.addEventListener('pop-playlist', e => { });

eventSource.addEventListener('pop-set-playlist', e => {
    const app = AppSingleton.get();
    const data = JSON.parse(e.data);

    app.player.playlist = app.player.playlist_stack.shift();
    app.player.playlist.current = data.position;

    app.updatePlaylist();

    // TODO: timestamp
});

eventSource.addEventListener('set-playlist-add-policy', e => {
    const app = AppSingleton.get();
    app.player.playlist.add_policy = JSON.parse(e.data);
});

function setPlayback(app, playback) {
    app.player.state = playback;
    app.handleStateChange();
    app.updatePlayBtn(app.isPlaying());
}
