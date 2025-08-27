const settingsTabs = document.querySelector('#settings .tabs');

const toggleTemplate = document.getElementById('toggle-setting');
const selectTemplate = document.getElementById('select-setting');
const listTemplate = document.getElementById('list-setting');
const listItemTemplate = document.getElementById('list-item-setting');
const inputTemplate = document.getElementById('input-setting');

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

    render() {
        const content = document.querySelector('#settings .settings-wrapper');

        settingsTabs.querySelectorAll('.tab').forEach((tab, i) => {
            if (i == 0) tab.classList.add('active');
            else tab.remove();
        });
        content.querySelectorAll('.settings-content').forEach((page, i) => {
            if (i == 0) page.classList.add('active');
            else page.remove();
        });

        const filler = settingsTabs.querySelector('.filler');
        configSchema.forEach((group, id) => {
            const page = document.createElement('div');
            page.classList.add('settings-content');

            for (const key in group.settings) {
                const setting =
                    this.getSettingElement(key, group.settings[key]);
                if (setting === null) continue;
                page.appendChild(setting);
            }
            content.appendChild(page);

            const tab = document.createElement('button');
            tab.classList.add('tab');
            tab.textContent = group.name;
            tab.onclick = () => Config.showPage(id + 1);
            settingsTabs.insertBefore(tab, filler);
        });
    }

    getSettingElement(key, setting) {
        switch (setting.type) {
            case "bool":
                return this.getToggleSetting(key, setting);
            case "string":
                return this.getInputSetting(key, setting);
            case "duration":
                return this.getDurationSetting(key, setting);
            case "float":
                return this.getFloatSetting(key, setting);
            case "select":
                return this.getSelectSetting(key, setting);
            case "list":
                return this.getListSetting(key, setting);
            default:
                return null;
        }
    }

    getToggleSetting(key, setting) {
        const clone = toggleTemplate.content.cloneNode(true);
        const toggle = clone.querySelector('.switch-setting');

        toggle.querySelector('.label').textContent = setting.label;
        toggle.querySelector('.description').textContent = setting.description;

        const input = toggle.querySelector('input');
        input.checked = this[key];
        input.onclick = () => this.toggleSwitch(key);

        return toggle;
    }

    getSelectSetting(key, setting) {
        const clone = selectTemplate.content.cloneNode(true);
        const select = clone.querySelector('.select-setting');

        select.querySelector('.label').textContent = setting.label;
        select.querySelector('.description').textContent = setting.description;

        const input = select.querySelector('select');
        for (const opt of setting.options) {
            const option = document.createElement('option');
            option.textContent = opt;
            input.appendChild(option);
        }

        input.addEventListener('change', () => {
            const value = input.value.replace(' ', '');
            this[key] = value;
        });

        return select;
    }

    getListSetting(key, setting) {
        const clone = listTemplate.content.cloneNode(true);
        const list = clone.querySelector('.list-setting');

        list.querySelector('.label').textContent = setting.label;
        list.querySelector('.description').textContent = setting.description;

        const items = list.querySelector('.items');
        const input = list.querySelector('.input-item');
        for (const item of this[key]) {
            const itemClone = listItemTemplate.content.cloneNode(true);
            itemClone.querySelector('p').textContent = item;
            items.insertBefore(itemClone, input);
        }

        return list;
    }

    getInputSetting(key, setting) {
        const clone = inputTemplate.content.cloneNode(true);
        const element = clone.querySelector('.input-setting');

        element.querySelector('.label').textContent = setting.label;
        element.querySelector('.description').textContent = setting.description;

        const input = element.querySelector('input');
        input.value = this[key];

        return element;
    }

    getFloatSetting(key, setting) {
        const element = this.getInputSetting(key, setting);
        const input = element.querySelector('input');
        input.addEventListener('input', () => {
            input.value =
                input.value.replace(/[^0-9.]/g, '').replace(/(\..*)\./g, '$1');
        })

        return element;
    }

    getDurationSetting(key, setting) {
        const regex = /^([0-9]*d)?([0-9]*:)?([0-9]*:)?([0-9]*|\.|[0-9]*\.[0-9]*|\.[0-9])$/;

        const element = this.getInputSetting(key, setting);
        const input = element.querySelector('input');
        input.addEventListener('input', () => {
            input.classList.remove('invalid');
            if (!regex.test(input.value.trim()))
                input.classList.add('invalid');
        });

        return element;
    }

    static showPage(id) {
        const tabs = settingsTabs.querySelectorAll('.tab');
        const pages = document.querySelectorAll('#settings .settings-content');

        for (let i = 0; i < tabs.length; i++) {
            const tab = tabs[i];
            const page = pages[i];

            tab.classList.remove('active');
            page.classList.remove('active');
            if (i === id) {
                tab.classList.add('active');
                page.classList.add('active');
            }
        }
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

const configSchema = [
    // Library settings section
    {
        name: "Library",
        settings: {
            search_paths: {
                type: "list",
                label: "Search paths",
                description: "List of paths where to search for music.",
                default: []
            },
            audio_extensions: {
                type: "list",
                label: "Audio extensions",
                description: "List of file extensions to load to library.",
                default: ["flac", "mp3", "m4a", "mp4"]
            },
            recursive_search: {
                type: "bool",
                label: "Recursive search",
                description: "Searches for songs in library paths recursively" +
                    "(traversing all subdirectories).",
                default: true
            },
            update_library_on_start: {
                type: "bool",
                label: "Update library on start",
                description: "Searches library paths every time on startup.",
                default: true
            },
            remove_missing_on_load: {
                type: "bool",
                label: "Remove missing on load",
                description: "Removes nonexisting songs from library.",
                default: true
            }
        }
    },
    // Playlists settings section
    {
        name: "Playlist",
        settings: {
            control_aliases: {
                type: "objectList",
                label: "Control aliases",
                description: "Alias is a set of control messages that will be" +
                    " used in its place.",
                default: []
            },
            default_playlist_end_action: {
                type: "string",
                label: "Default playlist end action",
                description: "Alias invocation set as playlist end action.",
                default: null
            },
            simple_sorting: {
                type: "bool",
                label: "Simple sorting",
                description: "Uamp sorts only by the given field.",
                default: false
            },
            shuffle_current: {
                type: "bool",
                label: "Shuffle current",
                description: "Shuffles all songs in the playlist, including " +
                    "current.",
                default: true
            }
        }
    },
    // Playback settings section
    {
        name: "Playback",
        settings: {
            play_on_start: {
                type: "bool",
                label: "Play on start",
                description: "Uamp will continue playing when it starts.",
                default: false
            },
            volume_jump: {
                type: "float",
                label: "Volume jump",
                description: "Default change of volume.",
                default: 2.5
            },
            save_playback_pos: {
                type: "select",
                label: "Save playback position",
                description: "Retaining position within current track after " +
                    "exiting",
                default: "OnClose",
                options: ["Never", "On Close", "Always"]
            },
            fade_play_pause: {
                type: "duration",
                label: "Fade play/pause",
                description: "Smoothly change volume when playing/pausing.",
                default: "00:00.15"
            },
            gapless: {
                type: "bool",
                label: "Gapless",
                description: "Removes silent parts between songs.",
                default: true
            },
            seek_jump: {
                type: "duration",
                label: "Seek jump",
                description: "Default seek amount when using fast forward and" +
                    "rewind.",
                default: "00:10"
            },
            previous_timeout: {
                type: "duration",
                label: "Previous timeout",
                description: "Jumping to previous song timeout.",
                default: null,
            }
        }
    },
    // Server settings section
    {
        name: "Server",
        settings: {
            server_address: {
                type: "string",
                label: "Server address",
                description: "Address of the backend HTTP server.",
                default: "127.0.0.1"
            },
            port: {
                type: "number",
                label: "Server port",
                description: "Port of the backend HTTP server.",
                default: 8267
            },
            system_player: {
                type: "bool",
                label: "System player",
                description: "Integrate with the system as media player.",
                default: true
            }
        }
    }
];
