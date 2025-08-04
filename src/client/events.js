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

eventSource.addEventListener('set-playlist', e => { });

eventSource.addEventListener('playback', e => {
    const app = AppSingleton.get();
    app.player.state = JSON.parse(e.data);
    app.updatePlayBtn(app.isPlaying());
});

eventSource.addEventListener('playlist-jump', e => {
    const app = AppSingleton.get();
    const jump = JSON.parse(e.data);

    app.player.state = jump.playback;
    app.updatePlayBtn(app.isPlaying());

    app.player.playlist.current = jump.position;

    app.updateCurrent(app.getPlaying());
});

eventSource.addEventListener('seek', e => { });

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

eventSource.addEventListener('pop-set-playlist', e => { });

eventSource.addEventListener('set-playlist-add-policy', e => { });
