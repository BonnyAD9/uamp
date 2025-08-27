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
    constructor(data, schema) {
        this.schema = schema;
        Object.assign(this, data);
    }

    static async init(data) {
        const schema = await fetch('assets/config_schema.json')
            .then(res => {
                if (!res.ok) throw new Error('failed to load config');
                return res.json();
            })
            .then(res => res.properties)
            .catch(_ => null);
        return new Config(data, schema);
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
        settingsGroups.forEach((group, id) => {
            const page = document.createElement('div');
            page.classList.add('settings-content');

            for (const key of group.settings) {
                const setting =
                    this.getSettingElement(key, this.schema[key]);
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
        // TODO: handle nullable
        if (Array.isArray(setting.type)) {
            const types = setting.type.filter(t => t !== "null");
            setting.type = types[0];
        }

        let element;
        switch (setting.type) {
            case "boolean":
                element = this.getToggleSetting(key);
                break;
            case "string":
                element = this.getInputSetting(key, setting);
                break;
            case "number":
                element = this.getFloatSetting(key, setting);
                break;
            // case "select":
            //     return this.getSelectSetting(key, setting);
            // case "list":
            //     return this.getListSetting(key, setting);
            default:
                return null;
        }

        element.querySelector('.label').textContent = Config.keyToLabel(key);
        element.querySelector('.description').textContent = setting.description;
        return element;
    }

    getToggleSetting(key) {
        const clone = toggleTemplate.content.cloneNode(true);
        const toggle = clone.querySelector('.switch-setting');

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
        console.log(key, this);
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

        const input = element.querySelector('input');
        input.value = this[key] ?? '';

        if (setting.pattern) {
            input.addEventListener('input', () => {
                const regex = new RegExp(setting.pattern);
                input.classList.remove('invalid');
                if (!regex.test(input.value))
                    input.classList.add('invalid');
            });
        }

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

    static keyToLabel(key) {
        const label = key.replaceAll('_', ' ');
        return label.charAt(0).toUpperCase() + label.slice(1);
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

const settingsGroups = [
    {
        name: "Library", settings: [
            "search_paths", "audio_extensions", "recursive_search",
            "update_library_on_start", "remove_missing_on_load"
        ]
    },
    {
        name: "Playlist", settings: [
            "control_aliases", "default_playlist_end_action", "simple_sorting",
            "shuffle_current"
        ]
    },
    {
        name: "Playback", settings: [
            "play_on_start", "volume_jump", "save_playback_pos",
            "fade_play_pause", "gapless", "seek_jump", "previous_timeout"
        ]
    },
    {
        name: "Server", settings: [
            "server_address", "port", "system_player"
        ]
    }
];
