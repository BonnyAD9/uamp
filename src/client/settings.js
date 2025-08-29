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
        this.update(name, !this[name]);
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
        if (setting.enum) {
            return this.getSelectSetting(key, setting);
        }

        // TODO: handle nullable
        if (Array.isArray(setting.type)) {
            const types = setting.type.filter(t => t !== "null");
            setting.type = types[0];
        }

        switch (setting.type) {
            case "boolean":
                return this.getToggleSetting(key, setting);
            case "string":
                return this.getInputSetting(key, setting);
            case "number":
                return this.getNumberSetting(key, setting, true);
            case "integer":
                return this.getNumberSetting(key, setting, false);
            // case "list":
            //     return this.getListSetting(key, setting);
            default:
                return null;
        }
    }

    /**
     * Gets the toggle setting element
     * @param {string} key - setting key
     * @param {*} setting - setting schema
     * @returns HTML toggle setting element
     */
    getToggleSetting(key, setting) {
        const clone = toggleTemplate.content.cloneNode(true);
        const toggle = clone.querySelector('.switch-setting');

        const input = toggle.querySelector('input');
        input.checked = this[key];
        input.onclick = () => this.toggleSwitch(key);

        Config.settingDetails(toggle, key, setting);
        return toggle;
    }

    /**
     * Gets the select setting element
     * @param {string} key - setting key
     * @param {*} setting - setting schema
     * @returns HTML select setting element
     */
    getSelectSetting(key, setting) {
        const clone = selectTemplate.content.cloneNode(true);
        const select = clone.querySelector('.select-setting');

        const input = select.querySelector('select');
        input.addEventListener('change', () => this.update(key, input.value));
        for (const opt of setting.enum) {
            const option = document.createElement('option');
            option.textContent = opt;
            option.value = opt;
            input.appendChild(option);
        }
        input.value = this[key];

        Config.settingDetails(select, key, setting);
        return select;
    }

    /**
     * Gets the text setting element
     * @param {string} key - setting key
     * @param {*} setting - setting schema
     * @returns HTML text setting element
     */
    getInputSetting(key, setting) {
        const element = this.#getGenericInput(key, '', setting);
        const input = element.querySelector('input');

        const regex = setting.pattern ? new RegExp(setting.pattern) : null;
        const isValid = val => regex ? regex.test(val) : true;
        const validate = () => {
            input.classList.toggle('invalid', !isValid(input.value))
        };

        input.addEventListener('input', validate);
        input.addEventListener('blur', () => {
            validate();
            if (input.value !== this[key] && isValid(input.value))
                this.update(key, input.value);
        });

        Config.settingDetails(element, key, setting);
        return element;
    }

    /**
     * Gets the number setting element
     * @param {string} key - setting key
     * @param {*} setting - setting schema
     * @param {boolean} float - whether the number is float or integer
     * @returns HTML number setting element
     */
    getNumberSetting(key, setting, float = true) {
        const element = this.#getGenericInput(key, 0.0, setting);
        const input = element.querySelector('input');

        const numfy = val => {
            if (!float)
                return val.replace(/[^0-9]/g, '');
            return val.replace(/[^0-9.]/g, '').replace(/(\..*)\./g, '$1');
        };

        const defaultValue = float ? 0.0 : 0;
        const parse = val => {
            const n = float ? parseFloat(val) : parseInt(val, 10);
            return isNaN(n) ? defaultValue : n;
        }

        input.addEventListener('input', () => input.value = numfy(input.value));
        input.addEventListener('blur', () => {
            input.value = numfy(input.value);
            const val = parse(input.value);
            if (val !== this[key])
                this.update(key, val);
        });

        return element;
    }

    getListSetting(key, setting) {
        const clone = listTemplate.content.cloneNode(true);
        const list = clone.querySelector('.list-setting');

        const items = list.querySelector('.items');
        const input = list.querySelector('.input-item');

        for (const item of this[key]) {
            const itemClone = listItemTemplate.content.cloneNode(true);
            itemClone.querySelector('p').textContent = item;
            items.insertBefore(itemClone, input);
        }

        Config.settingDetails(list, key, setting);
        return list;
    }

    #getGenericInput(key, defaultValue, setting) {
        const clone = inputTemplate.content.cloneNode(true);
        const element = clone.querySelector('.input-setting');

        const input = element.querySelector('input');
        input.value = this[key] ?? defaultValue;

        input.addEventListener('keydown', e => {
            if (e.key === 'Enter')
                input.blur();
            if (e.key === 'Escape') {
                input.value = this[key] ?? defaultValue;
                input.blur();
            };
        });

        Config.settingDetails(element, key, setting);
        return element;
    }

    update(key, value) {
        this[key] = value;
        fetch('/api/ctrl', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                SetConfig: { [key]: value }
            })
        });
    }

    static settingDetails(element, key, setting) {
        element.querySelector('.label').textContent = Config.keyToLabel(key);
        element.querySelector('.description').textContent = setting.description;
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
