import Api from "../../api.js";
import { onSongIconLoad } from "../colors.js";
import Duration from "../../helper/duration.js";
import Album from "../../library/album.js";

/**
 * @typedef {Object} BarControl
 * @property {string} id
 * @property {string} icon
 * @property {string} action
 */

/** @type {BarControl[]} */
const DEFAULT_CONTROLS = [
    { id: "prev", icon: "previous.svg", action: "ps" },
    { id: "play", icon: "play_pause.svg", action: "pp" },
    { id: "next", icon: "next.svg", action: "ns" },
];

export default class PlayerBar extends HTMLElement {
    constructor() {
        super();

        this.playing = false;

        this.radId = null;
        this.lastUpdate = 0;

        /** @type {Duration} */
        this.currentPos = null;
        /** @type {Duration} */
        this.totalPos = null;
    }

    connectedCallback() {
        const template = document.getElementById("bar-template");
        this.appendChild(template.content.cloneNode(true));

        // Prefetch all reusable DOM elements.
        this.controls = this.querySelector(".controls");
        this.slider = this.querySelector("#progressBar");
        this.timestampCur = this.querySelector("#timestamp-cur");
        this.timestampTotal = this.querySelector("#timestamp-total");
        this.playlist = this.querySelector(".playlist");
        this.songIcon = this.querySelector(".info .info-pic img");
        this.songTitle = this.querySelector(".info .title h3");
        this.songArtist = this.querySelector("#bar-artist");
        this.songAlbum = this.querySelector("#bar-album");
        this.barBackdrop = this.querySelector(".backdrop");
        this.volumeSlider = this.querySelector("#volumeSlider");
        this.volumeValue = this.querySelector("#volumeValue");
        this.volumeIcon = this.querySelector(".volume img");

        this.renderControls(DEFAULT_CONTROLS);
        this.playBtn = this.querySelector("#play");

        this.#setupListeners();
    }

    /** Toggles between expanded and normal player bar. */
    toggleBar() {
        this.classList.toggle("expanded");
        if (this.classList.contains("expanded")) {
            // TODO: dispatchEvent and then handle it
            setTimeout(() => app.createBarSongs(), 200);
        }
    }

    /**
     * Sets the playing state and updates related bar elements.
     * @param {bool} playing - true when playing, false otherwise
     */
    async setPlaying(playing) {
        if (this.playing === playing) return;
        this.playing = playing;

        this.#updatePlayBtn(playing);

        if (playing) {
            this.lastUpdate = performance.now();
            this.#startProgress();
        } else {
            this.#stopProgress();
        }
    }

    /**
     * Updates the bar elements displaying currently playing song.
     * @param {Song|null} song - currently playing song, display not playing
     * when null
     */
    updateCurrent(song) {
        this.songIcon.src = Album.getCover(song?.album_artist, song?.album);
        this.barBackdrop.src = Album.getCover(
            song?.album_artist,
            song?.album,
            64,
        );

        this.songTitle.textContent = song?.getTitle() ?? "Not Playing...";
        this.songArtist.textContent = song?.getArtists() ?? "";
        this.songAlbum.textContent = song?.getAlbum() ?? "";

        this.total = song?.length;
        this.timestampTotal.textContent = this.total?.format() ?? "-:--";
    }

    /**
     * Updates the timestamp bar components.
     * @param {Timestamp|null} time - new timestamp or null when none
     */
    updateTimestamp(time) {
        this.current = time?.current ?? this.current;
        this.total = time?.total ?? this.total;
        this.timestampCur.textContent = time?.current?.format() ?? "-:--";
        this.#displayProgress();
    }

    /**
     * Updates the volume bar components.
     * @param {number} volume - volume number in range 0 to 1
     * @param {bool} mute - whether volume is mutued or not
     */
    updateVolume(volume, mute = false) {
        const perVolume = Math.round(volume * 100);
        this.volumeSlider.value = perVolume;
        this.volumeValue.textContent = perVolume;

        const level = volume === 1.0 ? 4 : Math.ceil(volume * 3);
        let icon = `${mute ? "no_" : ""}volume_${level}.svg`;
        this.volumeIcon.src = `assets/svg/${icon}`;
    }

    /** Updates bar playlist gradient mask. */
    updatePlaylistMask() {
        const atTop = this.playlist.scrollTop === 0;
        const atBottom =
            this.playlist.scrollHeight - this.playlist.scrollTop ===
            this.playlist.clientHeight;

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

        this.playlist.style.webkitMaskImage = gradient;
        this.playlist.style.maskImage = gradient;
    }

    /**
     * Renders the bar controls.
     * @param {BarControl[]} controls - controls configuration
     */
    renderControls(controls) {
        this.controls.innerHTML = "";
        controls.forEach((ctrl) => {
            const btn = document.createElement("svg-icon");
            btn.setAttribute("src", `${ctrl.icon}`);
            btn.classList.add("control");

            if (ctrl.id) btn.id = ctrl.id;
            btn.dataset.action = ctrl.action;

            this.controls.appendChild(btn);
        });
    }

    /** Adds all required bar element listeners */
    #setupListeners() {
        this.songIcon.addEventListener("load", onSongIconLoad);

        this.controls.addEventListener("click", (e) => {
            const btn = e.target.closest(".control");
            if (btn?.dataset?.action) Api.ctrl(btn.dataset.action);
        });

        this.slider.addEventListener("input", () => {
            const pos = this.#getProgressPos()?.format();
            this.timestampCur.textContent = pos ?? "-:--";
            this.#stopProgress();
        });
        this.slider.addEventListener("change", () => {
            const pos = this.#getProgressPos()?.format();
            this.timestampCur.textContent = pos ?? "-:--";
            if (pos !== null) {
                Api.ctrl(`seek=${pos}`);
            }
            this.#tick();
        });

        this.volumeSlider.addEventListener("input", () => {
            Api.ctrl(`v=${this.volumeSlider.value / 100}`);
        });

        this.querySelector(".back-arrow").onclick = () => this.toggleBar();
        this.querySelector(".info").onclick = () => this.toggleBar();

        this.songAlbum.addEventListener("click", (e) =>
            this.#expandedAction(e, (song) =>
                app.albumBarClick(song.album_artist, song.album),
            ),
        );
        this.songArtist.addEventListener("click", (e) =>
            this.#expandedAction(e, (_) => app.artistBarClick(e)),
        );

        this.playlist.addEventListener("scroll", () => {
            this.updatePlaylistMask();
        });
        window.addEventListener("resize", () => this.updatePlaylistMask());
    }

    /** Helper function running action only when bar is expanded. */
    #expandedAction(e, action) {
        if (!this.classList.contains("expanded")) return;
        e.stopPropagation();
        action(app.player.getPlaying());
    }

    /** Runs the play button svg animation based on playing state. */
    async #updatePlayBtn(playing) {
        if (!this.playBtn) return;

        if (this.playBtn.waitRead) await this.playBtn.waitReady();

        const anim = playing ? "from_play_to_pause" : "from_pause_to_play";
        this.playBtn.triggerAnimation(anim);
    }

    /** Starts the progress bar animation. */
    #startProgress() {
        this.#stopProgress();
        this.radId = requestAnimationFrame(() => this.#tick());
    }

    /** Stops the progress bar animation. */
    #stopProgress() {
        if (this.rafId !== null) {
            cancelAnimationFrame(this.rafId);
            this.rafId = null;
        }
    }

    /** Ticks a progress bar. */
    #tick() {
        if (!this.playing) return;

        const now = performance.now();
        this.#displayProgress((now - this.lastUpdate) / 1000);
        this.lastUpdate = now;

        this.rafId = requestAnimationFrame(() => this.#tick());
    }

    /** Increments progress by given delta and display the progress. */
    #displayProgress(delta = 0) {
        const total = this.total?.toSecs() ?? 0;
        if (total === 0) {
            this.slider.value = 0;
            return;
        }

        let current = delta;
        if (this.current !== null) current += this.current.toSecs();

        this.slider.value = Math.min((current / total) * 100, 100);

        if (this.current !== null) {
            this.current.secs = Math.floor(current);
            this.current.nanos = Math.floor((current % 1) * 1e9);
            this.timestampCur.textContent = this.current.format();
        }
    }

    /**
     * Gets the progress bar duration.
     * @returns {Duration|null} - current progress bar duration
     */
    #getProgressPos() {
        const percent = this.slider.value / 100;
        return app.player.getPlaying()?.length?.fromPercent(percent);
    }
}
