const songsElement = document.querySelector('#library .songs tbody');
const playlistElement = document.querySelector('#playlist .songs tbody');
const songTemplate = document.getElementById('song-template');

const volumeSlider = document.getElementById('volumeSlider');
const volumeValue = document.getElementById('volumeValue');
const volumeIcon = document.querySelector('.volume img');
volumeSlider.addEventListener('input', () => {
    const value = volumeSlider.value;
    AppSingleton.get().updateVolume(value / 100);
    apiCtrl(`v=${value / 100}`);
});

class App {
    constructor(eventData) {
        this.library = eventData.library;
        this.player = eventData.player;
        this.position = eventData.position;

        console.log(this.player.state);
        console.log(this.isPlaying());
    }

    togglePlaying() {
        apiCtrl('pp').then(res => {
            if (res == 'Success!') {
                console.log(this.isPlaying());
                const playing = !this.isPlaying();
                this.player.state = playing ? 'Playing' : 'Paused';
                console.log(playing, this.player.state);
                this.updatePlayBtn(playing);
            }
        });
    }

    toggleMute() {
        apiCtrl('mute').then(res => {
            if (res == 'Success!') {
                this.player.mute = !this.player.mute;
                this.updateVolume(this.player.volume);
            }
        });
    }

    isPlaying() {
        return this.player.state === 'Playing';
    }

    getPlaying() {
        const playing = this.player.playlist.current;
        if (playing === null) return null;

        const current = this.player.playlist.songs[playing];
        return this.library.songs[current];
    }

    playSong(index) {
        console.log(index);
    }

    displaySongs() {
        const playing = this.player.playlist.current;
        const current = playing ? this.player.playlist.songs[playing] : null;

        songsElement.innerHTML = '';
        for (let i = 0; i < this.library.songs.length; i++) {
            const song = this.library.songs[i];
            if (song.deleted === true) continue;

            const row = this.getSongRow(song);
            if (i === current)
                row.classList.add('active');
            songsElement.appendChild(row);
        }
    }

    displayPlaylist() {
        const playing = this.player.playlist.current;
        playlistElement.innerHtml = '';
        for (let i = 0; i < this.player.playlist.songs.length; i++) {
            const song = this.library.songs[this.player.playlist.songs[i]];
            if (song.deleted === true) continue;

            const row = this.getSongRow(song);
            if (i === playing)
                row.classList.add('active');
            playlistElement.appendChild(row);
        }
    }

    updateAll() {
        this.updateCurrent(this.getPlaying());
        this.updateVolume(this.player.volume);
        this.updatePlayBtn(this.isPlaying());
    }

    formatDuration(duration) {
        const minutes = Math.floor(duration.secs / 60);
        const seconds = duration.secs % 60;
        return `${minutes}:${seconds.toString().padStart(2, '0')}`;
    }

    updatePlayBtn(playing) {
        const icon = playing ? 'play.svg' : 'pause.svg';
        document.getElementById('play').src = '/assets/svg/' + icon;
    }

    updateCurrent(song) {
        const title = document.querySelector('.info .title h3');
        if (song === null) {
            title.textContent = 'Not Playing';
            return;
        }
        title.textContent = song.title;
        document.querySelector('.info .title h4').textContent = song.artist;
    }

    updateVolume(volume) {
        this.player.volume = volume;
        const perVolume = Math.round(volume * 100);
        volumeSlider.value = perVolume;
        volumeValue.textContent = perVolume;

        const level = Math.ceil(volume * 4);
        let icon = `${this.player.mute ? 'no_' : ''}volume_${level}.svg`;
        volumeIcon.src = `/assets/svg/${icon}`;
    }

    getSongRow(song) {
        const cloned = songTemplate.content.cloneNode(true);
        const row = cloned.querySelector('tr');
        row.addEventListener('click', () => this.playSong(i));

        row.querySelector('.title').textContent = song.title;
        row.querySelector('.author').textContent = song.artist;
        row.querySelector('.album').textContent = song.album;
        row.querySelector('.year').textContent = song.year;
        row.querySelector('.length').textContent =
            this.formatDuration(song.length);
        row.querySelector('.genre').textContent = song.genre;

        return row;
    }
}

const navs = document.querySelectorAll('nav p');
const screens = document.querySelectorAll('.screen');

navs.forEach(item => {
    item.addEventListener('click', () => {
        navs.forEach(p => p.classList.remove('active'));
        item.classList.add('active');

        const targetId = item.dataset.screen;
        for (let screen of screens) {
            screen.classList.remove('active');
            if (screen.id == targetId) {
                screen.classList.add('active');
            }
        }
    });
});

function apiCtrl(query) {
    return fetch(`/api/ctrl?${query}`)
        .then(res => {
            if (!res.ok) {
                throw new Error(`HTTP error! status: ${res.status}`);
            }
            return res.text();
        }).catch(error => {
            console.error('Fetch error:', error);
        });
}
