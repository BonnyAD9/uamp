const settingsTabs = document.querySelector('#settings .tabs .tabs-wrapper');
const bar = document.querySelector('section.bar');

// Setting templates
const toggleTemplate = document.getElementById('toggle-setting');
const selectTemplate = document.getElementById('select-setting');
const inputTemplate = document.getElementById('input-setting');
const listTemplate = document.getElementById('list-setting');
const listItemTemplate = document.getElementById('list-item-setting');
const colorTemplate = document.getElementById('color-setting');

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

        const allKeys = Object.keys(this.schema).filter(k => k !== "$schema");
        const groupedKeys = settingsGroups.flatMap(group => group.settings);
        const otherKeys = allKeys.filter(k => !groupedKeys.includes(k));
        settingsGroups.push({ name: "Other", settings: otherKeys });

        for (let id = 0; id < settingsGroups.length; id++) {
            const group = settingsGroups[id];
            if (group.settings.length === 0) continue;

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
            settingsTabs.appendChild(tab);
        }
    }

    render_appearance() {

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
                return Config.getToggleSetting(key, this[key], setting,
                    () => this.update(key, !this[key])
                );
            case "string":
                return this.getInputSetting(key, setting);
            case "number":
                return this.getNumberSetting(key, setting, true);
            case "integer":
                return this.getNumberSetting(key, setting, false);
            case "array":
                return this.getListSetting(key, setting);
            default:
                return null;
        }
    }

    /**
     * Gets the toggle setting element
     * @param {string} key - setting key
     * @param {string} value - setting value
     * @param {*} setting - setting schema
     * @param {(event: MouseEvent) => void} onclick - click handler
     * @returns HTML toggle setting element
     */
    static getToggleSetting(key, value, setting, onclick) {
        const clone = toggleTemplate.content.cloneNode(true);
        const toggle = clone.querySelector('.switch-setting');

        const input = toggle.querySelector('input');
        input.checked = value;
        input.onclick = onclick;

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

    /**
     * Gets the list setting element
     * @param {string} key - setting key
     * @param {*} setting - setting schema
     * @returns HTML list setting element
     */
    getListSetting(key, setting) {
        const clone = listTemplate.content.cloneNode(true);
        const list = clone.querySelector('.list-setting');
        const items = list.querySelector('.items');
        const input = list.querySelector('.input-item');

        const addItem = (value) => {
            const itemEl = this.#getListItem(key, value);
            items.insertBefore(itemEl, input);
        }

        for (const item of this[key]) {
            addItem(item);
        }

        const inputEl = input.querySelector('input');
        const regex =
            setting.items.pattern ? new RegExp(setting.items.pattern) : null;
        const isValid = val => regex ? regex.test(val) : true;
        const validate = () => {
            inputEl.classList.toggle('invalid', !isValid(inputEl.value))
        };

        inputEl.addEventListener('input', validate);
        inputEl.addEventListener('keydown', e => {
            if (e.key === 'Enter')
                inputEl.blur();
            if (e.key === 'Escape') {
                inputEl.value = '';
                inputEl.blur();
            };
        });
        inputEl.addEventListener('blur', () => {
            validate();
            if (inputEl.value !== '' && isValid(inputEl.value)) {
                this[key].push(inputEl.value);
                addItem(inputEl.value);

                inputEl.value = '';
                this.update(key, this[key]);
            }
        });

        Config.settingDetails(list, key, setting);
        return list;
    }

    /**
     * Gets the toggle setting element
     * @param {string} key - setting key
     * @param {string} value - setting value
     * @param {*} setting - setting schema
     * @param {(e: InputEvent) => void} oninput - input handler
     * @returns HTML toggle setting element
     */
    static getColorSetting(key, value, setting, oninput) {
        const clone = colorTemplate.content.cloneNode(true);
        const color = clone.querySelector('.color-setting');

        const input = color.querySelector('input');
        input.value = value;
        input.oninput = oninput;

        Config.settingDetails(color, key, setting);
        return color;
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

    #getListItem(key, value) {
        const itemClone = listItemTemplate.content.cloneNode(true);
        const item = itemClone.querySelector('.item');
        item.querySelector('p').textContent = value;

        item.querySelector('img').addEventListener('click', () => {
            this[key] = this[key].filter(i => i !== value);
            item.remove();
            this.update(key, this[key]);
        });

        return item;
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

/**
 * Sets cookie with given name to given value
 * @param {string} name - name of the cookie
 * @param {*} value - cookie value
 * @param {integer} days - days the cookie is valid for
 */
function setCookie(name, value, days = 365) {
    const expires = new Date(Date.now() + days * 864e5).toUTCString();
    document.cookie = `${name}=${encodeURIComponent(value)}; ` +
        `expires=${expires}; path=/`;
}

/**
 * Gets cookie with the given name
 * @param {string} name - name of the cookie
 * @returns {?string} cookie value when found, else null
 */
function getCookie(name) {
    return document.cookie.split('; ').reduce((acc, part) => {
        const [k, v] = part.split('=');
        return k === name ? decodeURIComponent(v) : acc;
    }, null);
}

/// Displays appearance settings in its page
function displayAppearanceSettings() {
    const appearSettings = document.getElementById('appearanceSettings');

    const floatingBar = (getCookie('floatingBar') ?? 'true') === 'true';
    bar.classList.toggle('floating', floatingBar);
    const floatingBarToggle = Config.getToggleSetting(
        'floating_music_player_bar', floatingBar,
        { description: 'Music player bar detached from the window edges.' },
        e => {
            const floating = e.target.checked;
            bar.classList.toggle('floating', floating);
            setCookie('floatingBar', floating);
        }
    );
    appearSettings.appendChild(floatingBarToggle);

    const dynamicColor = (getCookie('dynamicColor') ?? 'true') === 'true';
    const dynamicColorToggle = Config.getToggleSetting(
        'dynamic_color', dynamicColor, {
        description: 'Change theme color based on the album art of the ' +
            'current song.'
    }, e => {
        const dynamic = e.target.checked;
        setCookie('dynamicColor', dynamic);
        setupDynamicColors(dynamic);
    });
    appearSettings.appendChild(dynamicColorToggle);

    const themeColor = getCookie('themeColor') ?? '#3acbaf';
    const themeColorPicker = Config.getColorSetting(
        'theme_color', themeColor, {
        description: 'Primary color used when dynamic color is off.'
    }, e => {
        const color = e.target.value;
        setCookie('themeColor', color);
        applyThemeColor(color);
    });
    appearSettings.appendChild(themeColorPicker);
}

displayAppearanceSettings();

// Config properties group mapping
var settingsGroups = [
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
            "server_address", "port", "enable_server", "skin", "system_player"
        ]
    },
    {
        name: "Update", settings: [
            "update_mode", "update_remote", "auto_restart"
        ]
    },
    {
        name: "Advanced", settings: [
            "library_path", "player_path", "cache_path"
        ]
    }
];
