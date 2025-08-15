const colorInput = document.getElementById('themeColor');
const savedColor = getCookie('themeColor') ?? '#3acbaf';
colorInput.value = savedColor;
applyThemeColor(savedColor);

colorInput.addEventListener('input', () => {
    const selectedColor = colorInput.value;
    applyThemeColor(selectedColor);
    setCookie('themeColor', selectedColor);
});

const floatingBarInput = document.getElementById('floatingBar');
const bar = document.querySelector('section.bar');
const floatingBarVal = getCookie('floatingBar') ?? 'true';
const floatingBar = floatingBarVal === 'true';
floatingBarInput.checked = floatingBar;
applyFloatingBar(floatingBar);

floatingBarInput.addEventListener('change', () => {
    const floating = floatingBarInput.checked;
    applyFloatingBar(floating);
    setCookie('floatingBar', floating);
});

class Config {
    constructor(data) {
        // Can be modified over HTTP only from localhost.
        this.library_path = data.library_path;
        this.player_path = data.player_path;
        this.cache_path = data.cache_path;
        this.search_paths = data.search_paths;
        this.audio_extensions = data.audio_extensions;
        this.recursive_search = data.recursive_search;
        this.server_address = data.server_address;
        this.port = data.port;
        this.skin = data.skin;
        this.update_mode = data.update_mode;
        this.update_remote = data.update_remote;
        this.delete_logs_after = data.delete_logs_after;
        this.enable_server = data.enable_server;
        this.autoauto_restartRestart = data.auto_restart;

        // Could be modified over HTTP.
        this.control_aliases = data.control_aliases;
        this.default_playlist_end_action = data.default_playlist_end_action;
        this.simple_sorting = data.simple_sorting;
        this.play_on_start = data.play_on_start;
        this.shuffle_current = data.shuffle_current;
        this.update_library_on_start = data.update_library_on_start;
        this.remove_missing_on_load = data.remove_missing_on_load;
        this.volume_jump = data.volume_jump;
        this.save_playback_pos = data.save_playback_pos;
        this.save_timeout = data.save_timeout;
        this.fade_play_pause = data.fade_play_pause;
        this.gapless = data.gapless
        this.seek_jump = data.seek_jump;
        this.client_image_lookup = data.client_image_lookup;
        this.system_player = data.system_player;
    }

    /**
     * Toggles switch value with the given name.
     * @param {string} name - name of the switch
     */
    toggleSwitch(name) {
        if (this[name] === undefined) return;
        this[name] = !this[name];
    }
}

function applyThemeColor(color) {
    document.documentElement.style.setProperty('--primary', color);
}

function applyFloatingBar(floating) {
    if (floating == true) {
        bar.classList.add('floating');
    } else {
        bar.classList.remove('floating');
    }
}

function setCookie(name, value, days = 365) {
    const expires = new Date(Date.now() + days * 864e5).toUTCString();
    document.cookie = `${name}=${encodeURIComponent(value)}; ` +
        `expires=${expires}; path=/`;
}

function getCookie(name) {
    return document.cookie.split('; ').reduce((acc, part) => {
        const [k, v] = part.split('=');
        return k === name ? decodeURIComponent(v) : acc;
    }, null);
}
