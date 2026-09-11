async function handleAdminLogin(event) {
    event.preventDefault();
    const email = document.getElementById('admin-email').value.trim();
    const password = document.getElementById('admin-password').value.trim();
    const errorEl = document.getElementById('error-msg');
    const btn = document.getElementById('loginBtn');

    errorEl.innerText = '';
    btn.innerText = 'Verifying Credentials...';
    btn.style.opacity = '0.7';

    try {
        const response = await fetch('/api/auth/login', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ email, password })
        });

        const data = await response.json();

        if (response.ok && data.success) {
            const role = (data.user?.role || data.role || '').toLowerCase();
            
            if (role === 'admin') {
                localStorage.setItem('aurawear_token', data.token || data.access_token);
                localStorage.setItem('user', JSON.stringify(data.user));
                window.location.href = '/admin.html';
            } else {
                errorEl.innerText = 'Access Denied: Administrative privileges required.';
                btn.innerText = 'Authenticate Session';
                btn.style.opacity = '1';
            }
        } else {
            errorEl.innerText = data.error || 'Authentication failed. Check your credentials.';
            btn.innerText = 'Authenticate Session';
            btn.style.opacity = '1';
        }
    } catch (err) {
        console.error('Login error:', err);
        errorEl.innerText = 'Server connection error. Please try again.';
        btn.innerText = 'Authenticate Session';
        btn.style.opacity = '1';
    }
}