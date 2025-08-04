const API_URL = 'http://localhost:8267/api';

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
