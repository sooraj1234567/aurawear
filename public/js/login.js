// =================================================================
// AURAWEAR — AUTHENTICATION LOGIC (login.js)
// =================================================================

function showLogin() {
  document.getElementById('loginBox').style.display = 'block';
  document.getElementById('regBox').style.display = 'none';
  document.getElementById('t1').classList.add('active');
  document.getElementById('t2').classList.remove('active');
  document.getElementById('msg').innerText = '';
}

function showReg() {
  document.getElementById('loginBox').style.display = 'none';
  document.getElementById('regBox').style.display = 'block';
  document.getElementById('t2').classList.add('active');
  document.getElementById('t1').classList.remove('active');
  document.getElementById('msg').innerText = '';
}

function getRedirectAndGo() {
  const redirect = localStorage.getItem('aurawear_redirect_after_login');
  if (redirect) {
    localStorage.removeItem('aurawear_redirect_after_login');
    location.href = redirect;
  } else {
    location.href = '/';
  }
}

async function doLogin() {
  const lemailInput = document.getElementById('lemail');
  const lpassInput = document.getElementById('lpass');
  const msgEl = document.getElementById('msg');

  const email = lemailInput.value.toLowerCase().trim();
  const password = lpassInput.value;

  if (!email || !password) {
    msgEl.innerText = 'Please enter both email and password.';
    return;
  }
  msgEl.style.color = 'var(--ink)';
  msgEl.innerText = 'Authenticating archive credentials...';

  try {
    const r = await fetch('/api/auth/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, password })
    });
    const d = await r.json();

    if (d.success) {
      const oldEmail = localStorage.getItem('aurawear_email');
      if (oldEmail && oldEmail !== email) {
        localStorage.removeItem('aurawear_wishlist');
        localStorage.removeItem('aurawear_cart');
      }
      localStorage.setItem('aurawear_token', d.token || email);
      localStorage.setItem('aurawear_email', email);

      if (!localStorage.getItem('aurawear_wishlist_' + email)) {
        localStorage.setItem('aurawear_wishlist_' + email, '[]');
      }
      if (!localStorage.getItem('aurawear_cart_' + email)) {
        localStorage.setItem('aurawear_cart_' + email, '[]');
      }
      getRedirectAndGo();
    } else {
      msgEl.style.color = '#a33';
      msgEl.innerText = d.error || 'Invalid email or password.';
    }
  } catch (e) {
    msgEl.style.color = '#a33';
    msgEl.innerText = 'Connection error. Ensure server is running.';
  }
}

async function doRegister() {
  const rnameInput = document.getElementById('rname');
  const remailInput = document.getElementById('remail');
  const rphoneInput = document.getElementById('rphone');
  const rpassInput = document.getElementById('rpass');
  const msgEl = document.getElementById('msg');

  const name = rnameInput.value.trim();
  const email = remailInput.value.toLowerCase().trim();
  const phone = rphoneInput.value.trim();
  const password = rpassInput.value;

  if (!name || !email || !password) {
    msgEl.innerText = 'Please fill out all required fields.';
    return;
  }
  msgEl.style.color = 'var(--ink)';
  msgEl.innerText = 'Creating archive profile...';

  try {
    const r = await fetch('/api/auth/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, password, name, first_name: name, last_name: "", phone })
    });
    const d = await r.json();

    if (d.success) {
      // Clear any leftover redirect intent so new users always land on the home archive
      localStorage.removeItem('aurawear_redirect_after_login');
      localStorage.removeItem('aurawear_wishlist');
      localStorage.removeItem('aurawear_cart');
      localStorage.setItem('aurawear_token', d.token || email);
      localStorage.setItem('aurawear_email', email);
      localStorage.setItem('aurawear_wishlist_' + email, '[]');
      localStorage.setItem('aurawear_cart_' + email, '[]');
      
      // Explicitly redirect new registrations to the Home Page (or change to '/profile.html' if you prefer)
      location.href = '/';
    } else {
      msgEl.style.color = '#a33';
      msgEl.innerText = d.error || 'Registration failed.';
    }
  } catch (e) {
    msgEl.style.color = '#a33';
    msgEl.innerText = 'Connection error. Ensure server is running.';
  }
}