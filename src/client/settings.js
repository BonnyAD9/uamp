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
const floatingBar = getCookie('floatingBar') ?? true;
floatingBarInput.checked = floatingBarInput;
applyFloatingBar(floatingBar);

floatingBarInput.addEventListener('change', () => {
    const floating = floatingBarInput.checked;
    applyFloatingBar(floating);
    setCookie('floatingBar', floating);
});

function applyThemeColor(color) {
    document.documentElement.style.setProperty('--primary', color);
}

function applyFloatingBar(floating) {
    if (floating) {
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
