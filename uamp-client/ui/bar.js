import Album from "../library/album.js";

const playBtn = document.getElementById("play");
/**
 * Updates the play button based on the playing state
 * @param {boolean} playing - whether player is playing or not
 */
export async function updatePlayBtn(playing) {
    if (playBtn.waitReady) {
        await playBtn.waitReady();
    }

    if (playing) {
        playBtn.triggerAnimation("from_play_to_pause_p1");
        playBtn.triggerAnimation("from_play_to_pause_p2");
    } else {
        playBtn.triggerAnimation("from_pause_to_play_p1");
        playBtn.triggerAnimation("from_pause_to_play_p2");
    }
}

const timestampCur = document.getElementById("timestamp-cur");
const timestampTotal = document.getElementById("timestamp-total");
/**
 * Update the current timestamp of the playing song
 * @param {Duration} time - current time
 */
export function updateTimestamp(time) {
    timestampCur.textContent = time.format();
}

const songIcon = document.querySelector(".bar .info .info-pic img");
const songTitle = document.querySelector(".bar .info .title h3");
const songArtist = document.getElementById("bar-artist");
const songAlbum = document.getElementById("bar-album");
const barBackdrop = document.querySelector(".bar .backdrop");
const slider = document.querySelector(".bar #progressBar");
/**
 * Updates the currently playing song info
 * @param {?Song} song - currently playing song
 */
export function updateCurrent(song) {
    songIcon.src = Album.getCover(song?.album_artist, song?.album);
    barBackdrop.src = Album.getCover(song?.album_artist, song?.album, 64);

    songTitle.textContent = song?.getTitle() ?? "Not Playing...";
    songArtist.textContent = song?.getArtists() ?? "";
    songAlbum.textContent = song?.getAlbum() ?? "";
    if (song?.album !== null)
        songAlbum.onclick = (e) => openAlbum(e, song.album_artist, song.album);

    timestampTotal.textContent = song?.length?.format() ?? "-:--";
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
        setTimeout(() => app.createBarSongs(), 200);
}
window.toggleBar = toggleBar;

function openAlbum(e, artist, album) {
    if (!bar.classList.contains("expanded")) return;
    e.stopPropagation();
    app.albumBarClick(artist, album);
}

songArtist.addEventListener("click", (e) => {
    if (!bar.classList.contains("expanded")) return;
    e.stopPropagation();
    app.artistBarClick(e);
});

const playlist = bar.querySelector(".playlist");
export function updatePlaylistMask() {
    const atTop = playlist.scrollTop === 0;
    const atBottom =
        playlist.scrollHeight - playlist.scrollTop === playlist.clientHeight;

    let gradient =
        "linear-gradient(to bottom, transparent, black 20%," +
        "black 80%, transparent)";

    if (atTop && atBottom) {
        gradient = "none";
    } else if (atTop) {
        gradient = "linear-gradient(to bottom, black 80%, transparent)";
    } else if (atBottom) {
        gradient = "linear-gradient(to bottom, transparent, black 20%)";
    }

    playlist.style.webkitMaskImage = gradient;
    playlist.style.maskImage = gradient;
}

playlist.addEventListener("scroll", updatePlaylistMask);
window.addEventListener("resize", updatePlaylistMask);
