import Album from "../library/album.js";

const pausePlayP1 = document.getElementById("from_pause_to_play_p1");
const playPauseP1 = document.getElementById("from_play_to_pause_p1");
const pausePlayP2 = document.getElementById("from_pause_to_play_p2");
const playPauseP2 = document.getElementById("from_play_to_pause_p2");
/**
 * Updates the play button based on the playing state
 * @param {boolean} playing - whether player is playing or not
 */
export function updatePlayBtn(playing) {
    if (playing) {
        playPauseP1.beginElement();
        playPauseP2.beginElement();
    } else {
        pausePlayP1.beginElement();
        pausePlayP2.beginElement();
    }
}

const songIcon = document.querySelector(".bar .info .info-pic img");
const songTitle = document.querySelector(".bar .info .title h3");
const songTitleExp = document.querySelector(".bar .info-title h3");
const songArtist = document.querySelector(".bar .info .title h4");
const songArtistExp = document.querySelector(".bar .info-title h4");
const barBackdrop = document.querySelector(".bar .backdrop");
/**
 * Updates the currently playing song info
 * @param {?Song} song - currently playing song
 */
export function updateCurrent(song) {
    if (song === null) {
        songTitle.textContent = "Not Playing...";
        songTitleExp.textContent = "Not Playing...";
        songArtist.textContent = "";
        songArtistExp.textContent = "";
        return;
    }

    songIcon.src = Album.getCover(song.artist, song.album);
    barBackdrop.src = Album.getCover(song.artist, song.album, 64);
    songTitle.textContent = song.title;
    songTitleExp.textContent = song.title;
    songArtist.textContent = song.artist;
    songArtistExp.textContent = song.artist;
}

const volumeSlider = document.getElementById("volumeSlider");
volumeSlider.addEventListener("input", () => {
    apiCtrl(`v=${volumeSlider.value / 100}`);
});

const volumeValue = document.getElementById("volumeValue");
const volumeIcon = document.querySelector(".volume img");
/**
 * Updates the volume UI elements
 * @param {number} volume - current playback volume
 * @param {boolean} mute - mute state
 */
export function updateVolume(volume, mute = false) {
    const perVolume = Math.round(volume * 100);
    volumeSlider.value = perVolume;
    volumeValue.textContent = perVolume;

    const level = volume === 1.0 ? 4 : Math.ceil(volume * 3);
    let icon = `${mute ? "no_" : ""}volume_${level}.svg`;
    volumeIcon.src = `assets/svg/${icon}`;
}

const bar = document.querySelector("section.bar");
export function toggleBar() {
    bar.classList.toggle("expanded");
    if (bar.classList.contains("expanded"))
        setTimeout(() => AppSingleton.get().createBarSongs(), 200);
}
window.toggleBar = toggleBar;
