const API_URL = 'http://localhost:8267/api';

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
    fetch(`${API_URL}/ctrl?${query}`)
        .then(res => {
            console.log('test');
            if (!res.ok) {
                throw new Error(`HTTP error! status: ${res.status}`);
            }
            return res.json();
        }).then(data => {
            console.log(data);
            if (data.error) {
                console.error(`API error: ${data.error}`);
            } else {
                console.log('API response:', data);
            }
        }).catch(error => {
            console.error('Fetch error:', error);
        });
}
