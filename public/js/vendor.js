// ==========================================
// --- VENDOR PORTAL CORE CONTROLLER ---
// ==========================================

let currentVendorId = "";
let vendorAccountStatus = "pending";
let editingProductId = null;
let productsCache = [];
let ordersCache = [];
let currentProductFilter = 'all';
let currentOrderFilter = 'all';
let vendorCategoryFees = [
    { name: 'Jackets', fee: 10 },
    { name: 'Outerwear', fee: 40 },
    { name: 'T-Shirts', fee: 15 },
    { name: 'Shoes', fee: 25 },
    { name: 'Dresses', fee: 20 },
    { name: 'Archive', fee: 10 },
    { name: 'Suits', fee: 30 },
    { name: 'Shirts', fee: 12 },
    { name: 'Tops', fee: 10 },
    { name: 'Bottoms', fee: 15 },
    { name: 'Accessories', fee: 8 }
];

document.addEventListener("DOMContentLoaded", () => {
    initNavigation();
    initVendorPortal();
    loadAdminCategories();
});

async function loadAdminCategories() {
    try {
        const savedFees = localStorage.getItem('marketplace_category_fees');
        if (savedFees) {
            vendorCategoryFees = JSON.parse(savedFees);
        } else {
            const res = await fetch('/api/categories');
            const data = await res.json();
            if (data.categories) {
                vendorCategoryFees = data.categories.map(c => typeof c === 'string' ? { name: c, fee: 10 } : c);
            }
        }
    } catch (e) {
        console.error('Using default category fees');
    }
    updateCategoryDropdown();
}

function updateCategoryDropdown(selectedVal = '') {
    const selectEl = document.getElementById('p_category');
    if (!selectEl) return;
    const activeCats = vendorCategoryFees.filter(c => c.status !== 'Disabled');
    selectEl.innerHTML = `<option value="">Select Category</option>` + (activeCats.length ? activeCats : vendorCategoryFees).map(c => `
        <option value="${c.name}" ${c.name === selectedVal ? 'selected' : ''}>${c.name} (Fee: $${c.fee})</option>
    `).join('');
    calculateVendorCustomerPricePreview();
}

function calculateVendorCustomerPricePreview() {
    const priceInput = document.getElementById('p_price');
    const catSelect = document.getElementById('p_category');
    const previewEl = document.getElementById('p-price-preview');
    if (!priceInput || !catSelect || !previewEl) return;

    const sellerPrice = parseFloat(priceInput.value) || 0;
    const selectedCatName = catSelect.value;
    const catObj = vendorCategoryFees.find(c => c.name === selectedCatName);
    const platformFee = catObj ? Number(catObj.fee) : 10;
    const customerPrice = sellerPrice + platformFee;

    previewEl.innerText = `Customer Price: $${customerPrice.toFixed(2)} (Your Earnings: $${sellerPrice.toFixed(2)} + Platform Fee: $${platformFee.toFixed(2)})`;
}

// --- TAB NAVIGATION SYSTEM ---
function initNavigation() {
    const navButtons = document.querySelectorAll('.nav-btn');
    const tabPanes = document.querySelectorAll('.tab-pane');

    navButtons.forEach(btn => {
        btn.addEventListener('click', (e) => {
            navButtons.forEach(b => b.classList.remove('active'));
            tabPanes.forEach(p => p.classList.remove('active'));

            const targetId = e.currentTarget.getAttribute('data-target');
            e.currentTarget.classList.add('active');
            
            const targetPane = document.getElementById(targetId);
            if(targetPane) {
                targetPane.classList.add('active');
            }
        });
    });
}

function extractId(val) {
    if (!val) return "";
    if (typeof val === 'string') return val;
    if (typeof val === 'number') return String(val);
    if (typeof val === 'object') {
        if (val.$oid) return val.$oid;
        if (val.id) return extractId(val.id);
        if (val._id) return extractId(val._id);
    }
    return String(val);
}

function findVendorId(data) {
    if (!data) return "";
    const u = data.user || data;
    return extractId(u._id) || extractId(u.id) || extractId(u.user_id) || extractId(u.vendor_id) || extractId(data._id);
}

// Strict validation helper to ensure an item belongs exclusively to the logged-in vendor
function itemBelongsToVendor(item, vendorId, storeName, storeEmail) {
    const itemVid = extractId(item.vendor_id || item.vendor || item.store_id || '').toLowerCase();
    const itemStore = String(item.vendor_name || item.store || item.storeName || '').toLowerCase();
    
    const vId = String(vendorId || '').toLowerCase();
    const vName = String(storeName || '').toLowerCase();
    const vEmail = String(storeEmail || '').toLowerCase();

    if (vId && itemVid && itemVid === vId) return true;
    if (vName && (itemStore === vName || itemVid === vName)) return true;
    if (vEmail && (itemStore === vEmail || itemVid === vEmail)) return true;
    
    return false;
}

async function initVendorPortal() {
    const token = localStorage.getItem('aurawear_token') || localStorage.getItem('token');
    if (!token) { window.location.href = '/login.html'; return; }

    try {
        const res = await fetch('/api/auth/me', {
            method: 'GET',
            headers: { 
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            }
        });
        
        let data;
        try {
            data = await res.json();
        } catch (err) {
            data = null;
        }

        if (res.ok && data && (data.success || data.user || data.id || data._id || data.email)) {
            const userObj = data.user || data;
            const userRole = (userObj.role || data.role || '').toLowerCase();
            const userEmail = (userObj.email || userObj.user?.email || '').toLowerCase();
            const userStoreName = (userObj.storeName || userObj.store || userObj.name || '').toLowerCase();
            
            if (userRole && userRole !== 'vendor' && userRole !== 'seller' && userRole !== 'admin') {
                alert('Access Denied: Customer accounts cannot access the vendor portal.');
                window.location.href = '/profile.html';
                return;
            }

            currentVendorId = findVendorId(data);
            if (!currentVendorId) {
                currentVendorId = userObj.id || userObj._id || userObj.vendor_id || userEmail || 'v_seller';
            }
            
            try {
                const statusRes = await fetch('/api/admin/sellers', {
                    headers: { 'Authorization': `Bearer ${token}` }
                });
                const statusData = await statusRes.json();
                const rawSellers = statusData.sellers || (Array.isArray(statusData) ? statusData : []);
                
                const matchedSeller = rawSellers.find(s => {
                    const sId = extractId(s.id || s._id).toLowerCase();
                    const sEmail = String(s.email || '').trim().toLowerCase();
                    return (currentVendorId && sId === currentVendorId.toLowerCase()) || (userEmail && sEmail === userEmail);
                });

                if (matchedSeller && matchedSeller.status) {
                    vendorAccountStatus = matchedSeller.status.toLowerCase();
                } else {
                    vendorAccountStatus = (userObj.status || data.status || 'active').toLowerCase();
                }
            } catch (err) {
                vendorAccountStatus = (userObj.status || data.status || 'active').toLowerCase();
            }

            try {
                const storedUsers = JSON.parse(localStorage.getItem('registeredUsers') || '[]');
                const matchedStoredUser = storedUsers.find(u => u.email && u.email.toLowerCase() === userEmail);
                if (matchedStoredUser && matchedStoredUser.status) {
                    vendorAccountStatus = matchedStoredUser.status.toLowerCase();
                }
            } catch (err) {}

            if (!currentVendorId) { window.location.href = '/login.html'; return; }
            
            const userName = userObj.storeName || userObj.store || userObj.name || userObj.first_name || 'Vendor Store';
            const displayEmail = userEmail || userObj.email || 'vendor@aurawear.com';

            document.getElementById('sidebarStoreName').innerText = userName;
            document.getElementById('displayStoreTitle').innerText = userName;
            document.getElementById('storeAvatarBadge').innerText = userName.substring(0, 2).toUpperCase();

            document.getElementById('settings-store-name').value = userName;
            document.getElementById('settings-store-email').value = displayEmail;
            document.getElementById('settings-store-id').value = currentVendorId;

            const noticeBanner = document.getElementById('vendorApprovalNotice');
            if (noticeBanner) {
                if (vendorAccountStatus !== 'active') {
                    noticeBanner.style.display = 'block';
                } else {
                    noticeBanner.style.display = 'none';
                }
            }

            loadVendorData(currentVendorId, userName, userEmail, userStoreName);
        } else {
            const localUser = JSON.parse(localStorage.getItem('user') || '{}');
            if (localUser && (localUser.email || localUser.id || localUser._id)) {
                currentVendorId = findVendorId(localUser) || localUser.email || 'v_seller';
                const userName = localUser.storeName || localUser.store || localUser.name || 'Vendor Store';
                const userEmail = localUser.email || 'vendor@aurawear.com';

                vendorAccountStatus = (localUser.status || 'active').toLowerCase();

                document.getElementById('sidebarStoreName').innerText = userName;
                document.getElementById('displayStoreTitle').innerText = userName;
                document.getElementById('storeAvatarBadge').innerText = userName.substring(0, 2).toUpperCase();

                document.getElementById('settings-store-name').value = userName;
                document.getElementById('settings-store-email').value = userEmail;
                document.getElementById('settings-store-id').value = currentVendorId;

                const noticeBanner = document.getElementById('vendorApprovalNotice');
                if (noticeBanner) {
                    if (vendorAccountStatus !== 'active') {
                        noticeBanner.style.display = 'block';
                    } else {
                        noticeBanner.style.display = 'none';
                    }
                }

                loadVendorData(currentVendorId, userName, userEmail, '');
            } else {
                window.location.href = '/login.html';
            }
        }
    } catch (e) {
        window.location.href = '/login.html';
    }
}

function checkVendorApprovalAndOpenModal() {
    if (vendorAccountStatus !== 'active') {
        alert('⚠️ Warning: Your store account is currently pending admin approval or has been suspended. You cannot list or upload new pieces until an administrator activates your store.');
        return;
    }
    openProductModal();
}

async function updateVendorProfile(e) {
    e.preventDefault();
    const token = localStorage.getItem('aurawear_token') || localStorage.getItem('token');
    const newName = document.getElementById('settings-store-name').value.trim();
    const newEmail = document.getElementById('settings-store-email').value.trim();
    const statusEl = document.getElementById('profileStatus');

    statusEl.style.color = 'var(--text-ink)';
    statusEl.innerText = 'Updating profile...';

    try {
        const res = await fetch('/api/auth/update', {
            method: 'POST',
            headers: { 
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            },
            body: JSON.stringify({ token, name: newName, email: newEmail })
        });
        const data = await res.json();

        if (data.success) {
            statusEl.style.color = '#2d6a4f';
            statusEl.innerText = '✓ Profile updated successfully!';
            document.getElementById('sidebarStoreName').innerText = newName;
            document.getElementById('displayStoreTitle').innerText = newName;
            document.getElementById('storeAvatarBadge').innerText = newName.substring(0, 2).toUpperCase();
        } else {
            statusEl.style.color = '#a85d35';
            statusEl.innerText = data.error || 'Failed to update profile.';
        }
    } catch (err) {
        statusEl.style.color = '#a85d35';
        statusEl.innerText = 'Connection error while saving profile.';
    }
}

async function loadVendorData(vendorId, storeName = '', storeEmail = '', storeKeyName = '') {
    if (!vendorId) return;
    
    let totalSellerEarnings = 0;
    let totalPlatformFees = 0;
    let totalGrossSales = 0;
    let activeCount = 0;
    let pendingCount = 0;

    try {
        const token = localStorage.getItem('aurawear_token') || localStorage.getItem('token');
        const pRes = await fetch(`/api/vendor/products?vendor_id=${encodeURIComponent(vendorId)}`, {
            headers: { 'Authorization': `Bearer ${token}` }
        });
        const pData = await pRes.json();
        let fetchedProducts = pData.products || (Array.isArray(pData) ? pData : []);

        productsCache = fetchedProducts.filter(p => itemBelongsToVendor(p, vendorId, storeName, storeEmail));

        activeCount = productsCache.length;
        document.getElementById('statInventory').innerText = activeCount;
        renderProductsTable();
    } catch (e) {
        document.getElementById('vendorProductList').innerHTML = `<tr><td colspan="8" class="empty-state" style="color:var(--rust);">Error loading inventory.</td></tr>`;
    }

    try {
        const token = localStorage.getItem('aurawear_token') || localStorage.getItem('token');
        const oRes = await fetch('/api/orders', {
            headers: { 'Authorization': `Bearer ${token}` }
        });
        const oData = await oRes.json();
        const rawOrders = oData.orders || (Array.isArray(oData) ? oData : []);
        
        ordersCache = rawOrders.filter(o => {
            const items = o.items || [];
            return items.some(i => itemBelongsToVendor(i, vendorId, storeName, storeEmail));
        });

        renderOrdersTable();

        ordersCache.forEach(o => {
            const items = o.items || [];
            const vendorItems = items.filter(i => itemBelongsToVendor(i, vendorId, storeName, storeEmail));

            const status = (o.status || 'pending').toLowerCase();
            if (status === 'pending' || status === 'processing') pendingCount++;

            vendorItems.forEach(i => {
                const qty = Number(i.qty || i.quantity || 1);
                const sPrice = Number(i.seller_price || i.price || i.unit_price) || 0;
                let pFee = Number(i.platform_fee || 0);
                if (!pFee && i.category) {
                    const foundCat = vendorCategoryFees.find(c => c.name.toLowerCase() === String(i.category).toLowerCase());
                    pFee = foundCat ? foundCat.fee : 10;
                } else if (!pFee) {
                    pFee = 10;
                }
                const cPrice = Number(i.customer_price || (sPrice + pFee));

                totalSellerEarnings += (sPrice * qty);
                totalPlatformFees += (pFee * qty);
                totalGrossSales += (cPrice * qty);
            });
        });

        document.getElementById('statRevenue').innerText = `$${vendorAccountStatus === 'active' ? totalSellerEarnings.toFixed(2) : '0.00'}`;
        document.getElementById('statOrders').innerText = pendingCount;

        document.getElementById('fin-gross').innerText = `$${vendorAccountStatus === 'active' ? totalGrossSales.toFixed(2) : '0.00'}`;
        document.getElementById('fin-fees').innerText = `$${vendorAccountStatus === 'active' ? totalPlatformFees.toFixed(2) : '0.00'}`;
        document.getElementById('fin-net').innerText = `$${vendorAccountStatus === 'active' ? totalSellerEarnings.toFixed(2) : '0.00'}`;

        if (vendorAccountStatus === 'active' && totalSellerEarnings > 0) {
            document.getElementById('finance-ledger-tbody').innerHTML = `
                <tr>
                    <td><b>PAY-${currentVendorId.substring(Math.max(0, currentVendorId.length - 5)).toUpperCase()}</b></td>
                    <td>Current Billing Cycle</td>
                    <td>$${totalGrossSales.toFixed(2)}</td>
                    <td style="color:var(--rust);">$${totalPlatformFees.toFixed(2)}</td>
                    <td><b>$${totalSellerEarnings.toFixed(2)}</b></td>
                    <td><span class="status-badge status-pending">PENDING</span></td>
                </tr>
            `;
        } else {
            document.getElementById('finance-ledger-tbody').innerHTML = `<tr><td colspan="6" class="empty-state">No settlement records found.</td></tr>`;
        }

    } catch (e) {
        document.getElementById('vendorOrderList').innerHTML = `<tr><td colspan="7" class="empty-state" style="color:var(--rust);">Error loading orders.</td></tr>`;
    }
}

function renderProductsTable() {
    const tbody = document.getElementById('vendorProductList');
    const searchInput = document.getElementById('product-search');
    const query = searchInput ? searchInput.value.toLowerCase() : '';

    const filtered = productsCache.filter(p => {
        const matchesStatus = currentProductFilter === 'all' || p.status === currentProductFilter;
        const matchesSearch = (p.name || '').toLowerCase().includes(query) || (p.category || '').toLowerCase().includes(query) || (p.gender || '').toLowerCase().includes(query);
        return matchesStatus && matchesSearch;
    });

    if (filtered.length === 0) {
        tbody.innerHTML = `<tr><td colspan="8" class="empty-state">No matching inventory items found.</td></tr>`;
        return;
    }

    tbody.innerHTML = filtered.map(p => {
        const prodId = extractId(p._id || p.id);
        const imgSrc = p.image || p.image_path || '/uploads/placeholder.png';
        const status = p.status || 'live';
        const displaySizes = Array.isArray(p.sizes) ? p.sizes.join(', ') : (p.size || 'M');
        const sPrice = Number(p.seller_price || p.price || 0);
        let pFee = Number(p.platform_fee || 0);
        if (!pFee && p.category) {
            const foundCat = vendorCategoryFees.find(c => c.name.toLowerCase() === String(p.category).toLowerCase());
            pFee = foundCat ? foundCat.fee : 10;
        } else if (!pFee) {
            pFee = 10;
        }
        const cPrice = Number(p.customer_price || (sPrice + pFee));

        return `
            <tr>
                <td><img src="${imgSrc}" style="width:36px; height:36px; object-fit:cover; border:1px solid var(--border);" onerror="this.style.display='none'"></td>
                <td><strong>${p.name || 'Untitled'}</strong></td>
                <td>${p.category || 'Archive'} • ${p.gender || 'Men'}</td>
                <td>${displaySizes}</td>
                <td>
                    Seller: $${sPrice.toFixed(2)}<br>
                    <span style="color:var(--gold); font-size:10px;">Fee: $${pFee.toFixed(2)}</span><br>
                    <b>Customer: $${cPrice.toFixed(2)}</b><br>
                    <span style="font-size:9px; opacity:0.6;">Stock: ${p.stock || 1}</span>
                </td>
                <td>${p.views || 42}</td>
                <td><span class="status-badge ${status === 'live' ? 'status-paid' : 'status-pending'}">${status.toUpperCase()}</span></td>
                <td>
                    <button onclick="startEditProduct('${prodId}')" style="background:#2d6a4f; color:#fff; border:none; padding:4px 8px; font-size:9px; cursor:pointer; border-radius:3px; margin-right:4px;">EDIT</button>
                    <button onclick="deleteProduct('${prodId}')" style="background:#a85d35; color:#fff; border:none; padding:4px 8px; font-size:9px; cursor:pointer; border-radius:3px;">REMOVE</button>
                </td>
            </tr>
        `;
    }).join('');
}

function renderOrdersTable() {
    const tbody = document.getElementById('vendorOrderList');
    const searchInput = document.getElementById('order-search');
    const query = searchInput ? searchInput.value.toLowerCase() : '';

    const storeName = document.getElementById('settings-store-name').value;
    const storeEmail = document.getElementById('settings-store-email').value;

    const filtered = ordersCache.filter(o => {
        const status = (o.status || 'pending').toLowerCase();
        const matchesStatus = currentOrderFilter === 'all' || status === currentOrderFilter;
        
        const orderIdStr = extractId(o._id || o.id).toLowerCase();
        const customerStr = (o.customer_name || o.name || o.email || '').toLowerCase();
        const matchesSearch = orderIdStr.includes(query) || customerStr.includes(query);

        return matchesStatus && matchesSearch;
    });

    if (filtered.length === 0) {
        tbody.innerHTML = `<tr><td colspan="7" class="empty-state">No incoming orders found matching criteria.</td></tr>`;
        return;
    }

    tbody.innerHTML = filtered.map(o => {
        const orderId = extractId(o._id || o.id);
        const status = (o.status || 'pending').toLowerCase();

        const items = o.items || [];
        const vendorItems = items.filter(i => itemBelongsToVendor(i, currentVendorId, storeName, storeEmail));

        let orderTotal = 0;
        vendorItems.forEach(i => { 
            const qty = Number(i.qty || i.quantity || 1);
            const sPrice = Number(i.seller_price || i.price || i.unit_price) || 0;
            let pFee = Number(i.platform_fee || 0);
            if (!pFee && i.category) {
                const foundCat = vendorCategoryFees.find(c => c.name.toLowerCase() === String(i.category).toLowerCase());
                pFee = foundCat ? foundCat.fee : 10;
            } else if (!pFee) {
                pFee = 10;
            }
            const cPrice = Number(i.customer_price || (sPrice + pFee));
            orderTotal += (cPrice * qty);
        });

        const customerName = o.customer_name || o.name || o.email || 'Collector';
        const paymentMethod = o.payment_status || o.payment_method || 'PAID';
        const orderDate = (o.created_at || o.date || '').substring(0, 10) || 'Recent';
        const itemsSummary = vendorItems.map(i => `${i.qty || 1}x ${i.name} (${i.size || 'M'})`).join(', ') || 'Archive Piece';

        let statusClass = 'status-pending';
        if (status === 'paid' || status === 'delivered') statusClass = 'status-paid';
        if (status === 'processing') statusClass = 'status-processing';
        if (status === 'shipped') statusClass = 'status-shipped';

        let actionButtons = `
            <button onclick="updateOrderStatus('${orderId}', 'processing')" style="background:#0077b6; color:#fff; border:none; padding:4px 8px; font-size:9px; cursor:pointer; border-radius:3px; margin-right:4px;">PROCESS</button>
            <button onclick="promptAndShipOrder('${orderId}')" style="background:#2d6a4f; color:#fff; border:none; padding:4px 8px; font-size:9px; cursor:pointer; border-radius:3px;">SHIP & TRACK</button>
        `;
        if (status === 'shipped') {
            const trackingInfo = o.tracking_number ? `<br><span style="font-size:9px; font-family:monospace;">Courier: ${o.courier || 'N/A'}<br>Tracking: ${o.tracking_number}</span>` : '';
            actionButtons = `<span style="font-size:10px; opacity:0.7;">Dispatched</span>${trackingInfo}`;
        }

        return `
            <tr>
                <td style="font-family:monospace; font-size:10px;">#AW-${orderId.substring(Math.max(0, orderId.length - 5)).toUpperCase()}</td>
                <td><strong>${customerName}</strong></td>
                <td><span class="status-badge active" style="font-size:9px;">${paymentMethod}</span></td>
                <td>${itemsSummary}</td>
                <td>$${orderTotal.toFixed(2)}</td>
                <td>${orderDate}</td>
                <td>
                    <div style="display:flex; flex-direction:column; gap:4px; align-items:flex-start;">
                        <span class="status-badge ${statusClass}">${status.toUpperCase()}</span>
                        <div>${actionButtons}</div>
                    </div>
                </td>
            </tr>
        `;
    }).join('');
}

function setProductFilter(btnElement) {
    document.querySelectorAll('#products .filter-btn').forEach(btn => btn.classList.remove('active'));
    btnElement.classList.add('active');
    currentProductFilter = btnElement.getAttribute('data-pstatus');
    renderProductsTable();
}

function filterVendorProducts() {
    renderProductsTable();
}

function setOrderFilter(btnElement) {
    document.querySelectorAll('#orders .filter-btn').forEach(btn => btn.classList.remove('active'));
    btnElement.classList.add('active');
    currentOrderFilter = btnElement.getAttribute('data-ostatus');
    renderOrdersTable();
}

function filterVendorOrders() {
    renderOrdersTable();
}

function addAngleInput(existingUrl = '') {
    const container = document.getElementById('anglesContainer');
    const div = document.createElement('div');
    div.className = 'angle-row';
    div.style.cssText = 'display:flex; gap:8px; align-items:center;';
    div.innerHTML = `
        ${existingUrl ? `<img src="${existingUrl}" style="width:35px; height:40px; object-fit:cover; border:1px solid var(--border-dark);">` : ''}
        <input type="file" accept="image/*" class="angle-file-input" style="padding:6px; background:#fff; flex:1; font-size:11px;">
        <input type="hidden" class="angle-url-input" value="${existingUrl}">
        <button type="button" onclick="this.parentElement.remove()" style="background:#a85d35; color:#fff; border:none; padding:8px 12px; font-size:10px; cursor:pointer; border-radius:3px;">✕</button>
    `;
    container.appendChild(div);
}

function openProductModal() {
    editingProductId = null;
    document.getElementById('mp-title').innerText = "List New Piece";
    document.getElementById('submitBtn').innerText = "PUBLISH TO STORE";
    document.getElementById('productForm').reset();
    document.getElementById('anglesContainer').innerHTML = '';
    
    document.querySelectorAll('input[name="product_size"]').forEach(cb => cb.checked = false);

    updateCategoryDropdown();
    addAngleInput();
    document.getElementById('productModal').classList.add('open');
}

function startEditProduct(id) {
    const product = productsCache.find(p => extractId(p._id || p.id) === String(id));
    if (!product) return;

    editingProductId = id;
    document.getElementById('mp-title').innerText = "Edit Piece (ID: " + id.substring(Math.max(0, id.length - 5)) + ")";
    document.getElementById('submitBtn').innerText = "SAVE CHANGES";

    document.getElementById('p_name').value = product.name || '';
    document.getElementById('p_gender').value = product.gender || 'Men';
    updateCategoryDropdown(product.category || '');
    
    let productSizes = [];
    if (Array.isArray(product.sizes) && product.sizes.length > 0) {
        productSizes = product.sizes.map(s => String(s).trim().toUpperCase());
    } else if (typeof product.sizes === 'string' && product.sizes.trim().length > 0) {
        productSizes = product.sizes.split(',').map(s => s.trim().toUpperCase());
    } else if (product.size && typeof product.size === 'string') {
        productSizes = product.size.split(',').map(s => s.trim().toUpperCase());
    }

    document.querySelectorAll('input[name="product_size"]').forEach(cb => {
        cb.checked = productSizes.includes(cb.value.trim().toUpperCase());
    });

    document.getElementById('p_price').value = product.seller_price || product.price || '';
    document.getElementById('p_stock').value = product.stock || 1;
    document.getElementById('p_desc').value = product.description || '';

    document.getElementById('anglesContainer').innerHTML = '';
    let gallery = Array.isArray(product.images) ? product.images : (product.image ? [product.image] : []);
    if (gallery.length === 0) gallery.push('');
    gallery.forEach(imgUrl => addAngleInput(imgUrl));

    calculateVendorCustomerPricePreview();
    document.getElementById('productModal').classList.add('open');
}

async function saveProduct(e) {
    e.preventDefault();
    if (!currentVendorId) return;

    const statusEl = document.getElementById('formStatus');
    statusEl.style.color = 'var(--text-ink)';
    statusEl.innerText = editingProductId ? 'Saving changes...' : 'Publishing to store...';

    try {
        const angleRows = document.querySelectorAll('.angle-row');
        let galleryImagePaths = [];
        const token = localStorage.getItem('aurawear_token') || localStorage.getItem('token');
        
        for (let row of angleRows) {
            const fileInput = row.querySelector('.angle-file-input');
            const hiddenUrl = row.querySelector('.angle-url-input');

            if (fileInput && fileInput.files[0]) {
                const formDa = new FormData();
                formDa.append('file', fileInput.files[0]);
                const upRes = await fetch('/api/upload', { 
                    method: 'POST', 
                    headers: { 'Authorization': `Bearer ${token}` },
                    body: formDa 
                });
                const upData = await upRes.json();
                const path = upData.file_path || upData.path || (upData.filename ? `/uploads/${upData.filename}` : "");
                if (path) galleryImagePaths.push(path);
            } else if (hiddenUrl && hiddenUrl.value.trim()) {
                galleryImagePaths.push(hiddenUrl.value.trim());
            }
        }

        const primaryImg = galleryImagePaths.length > 0 ? galleryImagePaths[0] : 'https://via.placeholder.com/150';

        const rawPriceInput = parseFloat(document.getElementById('p_price').value) || 0;
        const sellerPrice = parseFloat(rawPriceInput.toFixed(2));
        const category = document.getElementById('p_category').value;

        const catObj = vendorCategoryFees.find(c => c.name === category);
        const platformFee = catObj ? Number(catObj.fee) : 10;
        const customerPrice = sellerPrice + platformFee;

        const checkedSizes = Array.from(document.querySelectorAll('input[name="product_size"]:checked')).map(cb => cb.value);
        const finalSizeString = checkedSizes.length > 0 ? checkedSizes.join(', ') : 'M';

        const payload = {
            name: document.getElementById('p_name').value.trim(),
            gender: document.getElementById('p_gender').value,
            category: category,
            size: finalSizeString,
            sizes: checkedSizes.length > 0 ? checkedSizes : ['M'],
            seller_price: sellerPrice,
            platform_fee: platformFee,
            customer_price: customerPrice,
            price: customerPrice,
            stock: parseInt(document.getElementById('p_stock').value) || 1,
            image_path: primaryImg,
            image: primaryImg,
            images: galleryImagePaths,
            description: document.getElementById('p_desc').value.trim(),
            vendor_id: currentVendorId,
            status: 'live'
        };

        const endpoint = editingProductId ? `/api/products/${editingProductId}` : '/api/products';
        const method = editingProductId ? 'PUT' : 'POST';

        const res = await fetch(endpoint, {
            method: method,
            headers: { 
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            },
            body: JSON.stringify(payload)
        });
        const data = await res.json();

        if (data.success) {
            statusEl.style.color = '#2d6a4f';
            statusEl.innerText = editingProductId ? '✓ Changes saved successfully!' : '✓ Successfully listed to archive store!';
            forceCloseModal('productModal');
            loadVendorData(currentVendorId);
        } else {
            statusEl.style.color = '#a85d35';
            statusEl.innerText = data.error || 'Failed to save product.';
        }
    } catch (err) {
        statusEl.style.color = '#a85d35';
        statusEl.innerText = 'Server error during submission.';
    }
}

async function deleteProduct(id) {
    if (!confirm("Are you sure you want to remove this piece from the archive?")) return;
    const token = localStorage.getItem('aurawear_token') || localStorage.getItem('token');
    await fetch(`/api/products/${id}`, { 
        method: 'DELETE',
        headers: { 'Authorization': `Bearer ${token}` }
    });
    loadVendorData(currentVendorId);
}

async function promptAndShipOrder(orderId) {
    const courier = prompt("Enter Courier Name (e.g., FedEx, DHL, UPS):", "FedEx");
    if (courier === null) return;
    const trackingNumber = prompt("Enter Tracking Number:", "");
    if (trackingNumber === null) return;

    try {
        const token = localStorage.getItem('aurawear_token') || localStorage.getItem('token');
        const res = await fetch(`/api/orders/${orderId}/tracking`, {
            method: 'POST',
            headers: { 
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            },
            body: JSON.stringify({ courier, tracking_number: trackingNumber })
        });
        const data = await res.json();
        if (data.success) {
            alert('✓ Order marked as shipped and tracking info saved.');
            loadVendorData(currentVendorId);
        } else {
            alert(data.error || 'Failed to update tracking details.');
        }
    } catch (e) {
        alert('Server connection error while saving tracking.');
    }
}

async function updateOrderStatus(orderId, newStatus) {
    if (!confirm(`Update order status to ${newStatus.toUpperCase()}?`)) return;
    try {
        const token = localStorage.getItem('aurawear_token') || localStorage.getItem('token');
        const res = await fetch(`/api/orders/${orderId}/status`, {
            method: 'POST',
            headers: { 
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            },
            body: JSON.stringify({ status: newStatus })
        });
        const data = await res.json();
        if (data.success) {
            loadVendorData(currentVendorId);
        } else {
            alert(data.error || 'Failed to update status.');
        }
    } catch (e) {
        alert('Server connection error.');
    }
}

function closeModal(event, modalId) {
    if (event.target.id === modalId) {
        forceCloseModal(modalId);
    }
}

function forceCloseModal(modalId) {
    const modal = document.getElementById(modalId);
    if (modal) modal.classList.remove('open');
}

function vendorLogout() {
    localStorage.removeItem('token');
    localStorage.removeItem('aurawear_token');
    localStorage.removeItem('user');
    window.location.href = '/login.html';
}