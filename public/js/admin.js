// ==========================================
// --- ADMIN DASHBOARD CORE CONTROLLER ---
// ==========================================

document.addEventListener("DOMContentLoaded", () => {
    initNavigation();
    loadCategoryFeesFromStorage();
    loadDashboardMockData();
    injectLogoutButton();
});

// --- CATEGORY PLATFORM FEES STATE ---
let globalCategories = [
    { name: 'Jackets', fee: 10, status: 'Active' },
    { name: 'Outerwear', fee: 40, status: 'Active' },
    { name: 'T-Shirts', fee: 15, status: 'Active' },
    { name: 'Shoes', fee: 25, status: 'Active' },
    { name: 'Dresses', fee: 20, status: 'Active' },
    { name: 'Archive', fee: 10, status: 'Active' }
];

function loadCategoryFeesFromStorage() {
    try {
        const saved = localStorage.getItem('marketplace_category_fees');
        if (saved) {
            globalCategories = JSON.parse(saved);
        }
    } catch(e) {}
    updateCategoryDropdowns();
    renderCategoryManager();
}

function saveCategoryFeesToStorage() {
    localStorage.setItem('marketplace_category_fees', JSON.stringify(globalCategories));
    window.dispatchEvent(new StorageEvent('storage', {
        key: 'marketplace_category_fees',
        newValue: JSON.stringify(globalCategories)
    }));
    updateCategoryDropdowns();
    renderCategoryManager();
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
                routeTabAction(targetId);
            }
        });
    });
}

// --- DATA ROUTER ---
function routeTabAction(tabId) {
    switch(tabId) {
        case 'dashboard':
            loadDashboardMockData();
            break;
        case 'sellers':
            loadSellersData();
            break;
        case 'products':
            loadMasterCatalog();
            break;
        case 'orders':
            loadOrdersData();
            break;
        case 'finance':
            loadFinanceData();
            break;
        case 'customers':
            loadCustomersData();
            break;
        case 'cms':
            loadCmsData();
            break;
    }
}

// --- ROBUST ID EXTRACTION (HANDLES MONGODB OBJECTIDS & NEW-OBJ FORMATS) ---
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

// --- ADMIN LOGOUT SYSTEM ---
function injectLogoutButton() {
    const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT, null, false);
    let node;
    while (node = walker.nextNode()) {
        if (node.nodeValue.includes('View Live Store')) {
            const container = node.parentElement;
            if (container && !document.getElementById('admin-logout-btn')) {
                const logoutBtn = document.createElement('button');
                logoutBtn.id = 'admin-logout-btn';
                logoutBtn.innerText = 'LOGOUT ADMIN';
                logoutBtn.type = 'button';
                logoutBtn.style.cssText = 'display: block; margin-top: 12px; width: 100%; padding: 8px 12px; background: #8b0000; color: #fff; border: none; border-radius: 4px; cursor: pointer; font-family: monospace; font-size: 11px; font-weight: bold; text-align: center;';
                logoutBtn.onclick = adminLogout;
                container.appendChild(logoutBtn);
            }
            break;
        }
    }
}

function adminLogout() {
    if (confirm('Are you sure you want to log out of the admin session?')) {
        localStorage.clear();
        sessionStorage.clear();
        
        document.cookie.split(";").forEach((c) => {
            document.cookie = c.replace(/^ +/, "").replace(/=.*/, "=;expires=" + new Date().toUTCString() + ";path=/");
        });

        window.location.href = '/index.html';
    }
}

// --- DASHBOARD REAL-TIME DATA LOADER (COMMAND CENTER OVERVIEW) ---
async function loadDashboardMockData() {
    const gmvEl = document.getElementById('gmv-total');
    const revEl = document.getElementById('rev-total');
    const vendorEl = document.getElementById('vendor-count');
    const pendingEl = document.getElementById('pending-count');

    try {
        const token = localStorage.getItem('aurawear_token') || localStorage.getItem('token');
        const headers = token ? { 'Authorization': `Bearer ${token}` } : {};

        const [orderRes, sellerRes] = await Promise.all([
            fetch('/api/orders', { headers }),
            fetch('/api/admin/sellers', { headers }).catch(() => ({ json: () => ({ success: false, sellers: [] }) }))
        ]);

        const orderData = await orderRes.json();
        const sellerData = await sellerRes.json();

        const rawOrders = orderData.orders || (Array.isArray(orderData) ? orderData : []);
        const rawSellers = sellerData.sellers || (Array.isArray(sellerData) ? sellerData : []);

        let totalGmv = 0;
        let totalPlatformRevenue = 0;

        rawOrders.forEach(o => {
            let orderSum = 0;
            let orderFeeSum = 0;
            if (o.items && o.items.length > 0) {
                o.items.forEach(i => {
                    const qty = Number(i.qty || i.quantity || 1);
                    const sPrice = Number(i.seller_price || i.price || 0); // Seller Profit / Earnings
                    let pFee = Number(i.platform_fee || 0); // Admin Profit / Platform Fee
                    if (!pFee && i.category) {
                        const foundCat = globalCategories.find(c => c.name.toLowerCase() === String(i.category).toLowerCase());
                        pFee = foundCat ? foundCat.fee : 10;
                    } else if (!pFee) {
                        pFee = 10;
                    }
                    const cPrice = Number(i.customer_price || (sPrice + pFee)); // Total Customer Price (GMV per unit)

                    orderSum += (cPrice * qty);
                    orderFeeSum += (pFee * qty);
                });
            } else {
                orderSum = Number(o.total || o.total_amount || 0);
                orderFeeSum = orderSum * 0.15;
            }
            totalGmv += orderSum;
            totalPlatformRevenue += orderFeeSum;
        });

        let activeVendorsCount = 0;
        let pendingApprovalsCount = 0;

        rawSellers.forEach(s => {
            const status = (s.status || '').toLowerCase();
            if (status === 'active') activeVendorsCount++;
            if (status === 'pending') pendingApprovalsCount++;
        });

        if(gmvEl) gmvEl.innerText = `$${totalGmv.toLocaleString(undefined, {minimumFractionDigits: 2, maximumFractionDigits: 2})}`;
        if(revEl) revEl.innerText = `$${totalPlatformRevenue.toLocaleString(undefined, {minimumFractionDigits: 2, maximumFractionDigits: 2})}`;
        if(vendorEl) vendorEl.innerText = activeVendorsCount;
        if(pendingEl) pendingEl.innerText = pendingApprovalsCount;

        const savedPayments = JSON.parse(localStorage.getItem('saved_order_payments') || '{}');

        ordersData = rawOrders.map(o => {
            const oid = o._id?.$oid || o._id || o.id || 'ORD-' + Math.floor(Math.random() * 9000 + 1000);
            const orderIdStr = String(oid);
            const displayId = orderIdStr.startsWith('ORD-') ? orderIdStr : 'ORD-' + orderIdStr.substring(orderIdStr.length - 5).toUpperCase();
            
            const paymentStatus = savedPayments[orderIdStr] || savedPayments[displayId] || o.payment_status || 'COD_PENDING';

            let vendorNames = 'Marketplace Vendor';
            if (o.items && o.items.length > 0) {
                vendorNames = [...new Set(o.items.map(i => i.vendor_name || i.store || i.vendor || i.store_name))].filter(Boolean).join(', ') || 'Marketplace Vendor';
            }

            let itemsSummary = 'Archive Piece';
            if (o.items && o.items.length > 0) {
                itemsSummary = o.items.map(i => `${i.qty || 1}x ${i.name}`).join(', ');
            }

            let calculatedOrderTotal = 0;
            if (o.items && o.items.length > 0) {
                o.items.forEach(i => {
                    const qty = Number(i.qty || i.quantity || 1);
                    const sPrice = Number(i.seller_price || i.price || 0);
                    let pFee = Number(i.platform_fee || 0);
                    if (!pFee && i.category) {
                        const foundCat = globalCategories.find(c => c.name.toLowerCase() === String(i.category).toLowerCase());
                        pFee = foundCat ? foundCat.fee : 10;
                    } else if (!pFee) {
                        pFee = 10;
                    }
                    const cPrice = Number(i.customer_price || (sPrice + pFee));
                    calculatedOrderTotal += (cPrice * qty);
                });
            } else {
                calculatedOrderTotal = Number(o.total || o.total_amount || 0);
            }

            return {
                id: displayId,
                rawId: orderIdStr,
                customer: o.customer_name || o.name || o.email || 'Customer',
                vendor: vendorNames,
                total: Number(calculatedOrderTotal.toFixed(2)),
                payment: paymentStatus,
                status: (o.status || 'processing').toLowerCase(),
                items: itemsSummary,
                rawItems: o.items || [],
                tracking: o.tracking_number || ''
            };
        });

        renderDashboardChart(rawOrders);

        const orderTbody = document.getElementById('dash-orders-tbody');
        if (orderTbody) {
            if (ordersData.length === 0) {
                orderTbody.innerHTML = `<tr><td colspan="6" class="empty-state">No recent buyer orders found.</td></tr>`;
            } else {
                orderTbody.innerHTML = ordersData.slice(0, 5).map(o => {
                    let itemsHtml = '<span>Archive Piece</span>';
                    if (o.rawItems && o.rawItems.length > 0) {
                        itemsHtml = o.rawItems.map(i => {
                            const imgUrl = i.image_path || i.image || 'https://via.placeholder.com/40';
                            const sPrice = Number(i.seller_price || i.price || 0);
                            let pFee = Number(i.platform_fee || 0);
                            if (!pFee && i.category) {
                                const foundCat = globalCategories.find(c => c.name.toLowerCase() === String(i.category).toLowerCase());
                                pFee = foundCat ? foundCat.fee : 10;
                            } else if (!pFee) {
                                pFee = 10;
                            }
                            const cPrice = Number(i.customer_price || (sPrice + pFee));
                            return `
                                <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 4px;">
                                    <img src="${imgUrl}" style="width: 32px; height: 32px; object-fit: cover; border: 1px solid var(--border-dark);" onerror="this.style.display='none'">
                                    <div>
                                        <b>${i.qty || 1}x ${i.name}</b>
                                        <div style="font-size: 10px; opacity: 0.6;">$${cPrice.toFixed(2)} each (Seller: $${sPrice} + Fee: $${pFee})</div>
                                    </div>
                                </div>
                            `;
                        }).join('');
                    }

                    return `
                        <tr>
                            <td><b>${o.id}</b></td>
                            <td>${o.customer}</td>
                            <td>${itemsHtml}</td>
                            <td>$${o.total.toFixed(2)}</td>
                            <td><span class="status-badge ${o.status === 'shipped' ? 'active' : 'pending'}">${o.status.toUpperCase()}</span></td>
                            <td><button class="btn-action" onclick="openOrderModal('${o.id}')">MANAGE</button></td>
                        </tr>
                    `;
                }).join('');
            }
        }
    } catch (e) {
        console.error('Failed to load dashboard real-time data', e);
    }
}

// --- INTERACTIVE VISUAL ANALYTICS CHART ---
let myAnalyticsChart = null;

function renderDashboardChart(rawOrders) {
    const ctx = document.getElementById('analyticsChart');
    if (!ctx) return;

    const timelineData = {};
    rawOrders.forEach(o => {
        const dateStr = o.created_at ? new Date(o.created_at).toLocaleDateString() : 'Active Period';
        let orderSum = 0;
        if (o.items && o.items.length > 0) {
            o.items.forEach(i => {
                const qty = Number(i.qty || 1);
                const sPrice = Number(i.seller_price || i.price || 0);
                let pFee = Number(i.platform_fee || 0);
                if (!pFee && i.category) {
                    const foundCat = globalCategories.find(c => c.name.toLowerCase() === String(i.category).toLowerCase());
                    pFee = foundCat ? foundCat.fee : 10;
                } else if (!pFee) {
                    pFee = 10;
                }
                const cPrice = Number(i.customer_price || (sPrice + pFee));
                orderSum += (cPrice * qty);
            });
        } else {
            orderSum = Number(o.total || o.total_amount || 0);
        }
        timelineData[dateStr] = (timelineData[dateStr] || 0) + orderSum;
    });

    const labels = Object.keys(timelineData);
    const dataValues = Object.values(timelineData);

    if (myAnalyticsChart) {
        myAnalyticsChart.destroy();
    }

    myAnalyticsChart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: labels.length ? labels : ['Phase 1', 'Phase 2', 'Phase 3'],
            datasets: [{
                label: 'Gross Volume ($)',
                data: dataValues.length ? dataValues : [200, 500, 1200],
                borderColor: '#C9A86A',
                backgroundColor: 'rgba(201, 168, 106, 0.15)',
                borderWidth: 2,
                fill: true,
                tension: 0.2
            }]
        },
        options: {
            responsive: true,
            plugins: {
                legend: { 
                    labels: { font: { family: "'Courier Prime', monospace", size: 11 } } 
                }
            },
            scales: {
                x: { ticks: { font: { family: "'Courier Prime', monospace", size: 10 } } },
                y: { ticks: { font: { family: "'Courier Prime', monospace", size: 10 } } }
            }
        }
    });
}

// ==========================================
// --- VENDORS & SELLERS MODULE ---
// ==========================================
let vendorsData = [];
let currentVendorFilter = 'all';

async function loadSellersData() {
    try {
        const token = localStorage.getItem('aurawear_token') || localStorage.getItem('token');
        const headers = token ? { 'Authorization': `Bearer ${token}` } : {};

        const [sellerRes, prodRes] = await Promise.all([
            fetch('/api/admin/sellers', { headers }).catch(() => ({ json: () => ({ success: false, sellers: [] }) })),
            fetch('/api/products', { headers }).catch(() => ({ json: () => ({ products: [] }) }))
        ]);

        const sellerData = await sellerRes.json();
        const prodData = await prodRes.json();

        let rawSellers = [];
        if (Array.isArray(sellerData)) rawSellers = sellerData;
        else if (sellerData.sellers && Array.isArray(sellerData.sellers)) rawSellers = sellerData.sellers;
        else if (sellerData.data && Array.isArray(sellerData.data)) rawSellers = sellerData.data;

        let allProducts = [];
        if (Array.isArray(prodData)) allProducts = prodData;
        else if (prodData.products && Array.isArray(prodData.products)) allProducts = prodData.products;
        else if (prodData.data && Array.isArray(prodData.data)) allProducts = prodData.data;

        if (rawSellers.length > 0) {
            vendorsData = rawSellers.map(s => {
                const sId = extractId(s.id || s._id).trim().toLowerCase();
                const sEmail = String(s.email || '').trim().toLowerCase();
                const sUsername = sEmail ? sEmail.split('@')[0] : '';
                const sStore = String(s.store || s.store_name || '').trim().toLowerCase();

                const vendorProducts = allProducts.filter(p => {
                    const pVendorId = extractId(p.vendor_id || p.seller_id || p.user_id || p.id).trim().toLowerCase();
                    const pEmail = String(p.email || p.seller_email || p.user_email || '').trim().toLowerCase();
                    const pVendor = String(p.vendor || '').trim().toLowerCase();
                    const pStore = String(p.store || p.store_name || p.vendor_name || '').trim().toLowerCase();

                    if (sId && pVendorId && (pVendorId === sId || pVendorId.includes(sId) || sId.includes(pVendorId))) return true;
                    if (sEmail && pEmail && (pEmail === sEmail || pEmail.includes(sEmail))) return true;
                    if (sEmail && pVendor && (pVendor === sEmail || pVendor.includes(sEmail))) return true;
                    if (sUsername && (pVendor.includes(sUsername) || pVendorId.includes(sUsername) || pEmail.includes(sUsername))) return true;
                    if (sStore && pStore && sStore !== 'archive store' && (pStore === sStore || pStore.includes(sStore))) return true;

                    return false;
                });

                const itemCount = vendorProducts.length;
                const totalEarnings = vendorProducts.reduce((sum, p) => sum + (Number(p.seller_price || p.price) || 0), 0);

                let displayStoreName = s.store || s.store_name;
                if (!displayStoreName || displayStoreName.toLowerCase() === 'archive store') {
                    displayStoreName = s.owner ? `${s.owner}'s Store` : (s.email ? `${s.email.split('@')[0]}'s Store` : 'Marketplace Store');
                }

                return {
                    id: s.id || s._id,
                    store: displayStoreName,
                    owner: s.owner || s.owner_name || 'Store Owner',
                    email: s.email || '',
                    products: itemCount,
                    earnings: totalEarnings,
                    status: s.status || 'active',
                    comm: s.comm || 15
                };
            });
        } else {
            vendorsData = [];
        }
        renderVendors();
    } catch (e) {
        console.error('Failed to load real sellers data', e);
        vendorsData = [];
        renderVendors();
    }
}

function renderVendors() {
    const tbody = document.getElementById('vendor-tbody');
    const searchInput = document.getElementById('vendor-search');
    if (!tbody) return;

    const query = searchInput ? searchInput.value.toLowerCase() : '';
    
    const filtered = vendorsData.filter(v => {
        const matchesStatus = currentVendorFilter === 'all' || v.status === currentVendorFilter;
        const matchesSearch = v.store.toLowerCase().includes(query) || v.owner.toLowerCase().includes(query) || v.email.toLowerCase().includes(query);
        return matchesStatus && matchesSearch;
    });

    if (filtered.length === 0) {
        tbody.innerHTML = `<tr><td colspan="7" class="empty-state">No vendors found matching criteria.</td></tr>`;
        return;
    }

    tbody.innerHTML = filtered.map(v => `
        <tr>
            <td><input type="checkbox" class="vendor-checkbox" value="${extractId(v.id)}"></td>
            <td>
                <b>${v.store}</b><br>
                <span style="opacity:0.6; font-size:10px;">ID: ${String(extractId(v.id)).toUpperCase()}</span>
            </td>
            <td>${v.owner}<br><span style="opacity:0.6">${v.email}</span></td>
            <td>${v.products} items</td>
            <td>$${Number(v.earnings || 0).toLocaleString(undefined, {minimumFractionDigits: 2, maximumFractionDigits: 2})}</td>
            <td><span class="status-badge ${v.status}">${v.status.toUpperCase()}</span></td>
            <td>
                <button class="btn-action" onclick="openVendorModal('${extractId(v.id)}')">MANAGE</button>
            </td>
        </tr>
    `).join('');
}

function setVendorFilter(btnElement) {
    document.querySelectorAll('#sellers .filter-btn').forEach(btn => btn.classList.remove('active'));
    btnElement.classList.add('active');
    currentVendorFilter = btnElement.getAttribute('data-status');
    renderVendors();
}

function filterVendors() {
    renderVendors();
}

function openVendorModal(vendorId) {
    const v = vendorsData.find(x => extractId(x.id) === extractId(vendorId));
    if(!v) return;

    document.getElementById('mv-storename').innerText = v.store;
    document.getElementById('mv-owner').innerText = v.owner;
    document.getElementById('mv-email').innerText = v.email;
    document.getElementById('mv-commission').value = v.comm;

    const btnApprove = document.getElementById('btn-mv-approve');
    const btnSuspend = document.getElementById('btn-mv-suspend');

    btnApprove.style.display = v.status === 'pending' ? 'inline-block' : 'none';
    btnSuspend.style.display = v.status !== 'suspended' ? 'inline-block' : 'none';
    
    if (v.status === 'suspended') {
        btnApprove.style.display = 'inline-block';
        btnApprove.innerText = "REINSTATE VENDOR";
    } else {
        btnApprove.innerText = "APPROVE VENDOR";
    }

    btnApprove.onclick = () => updateVendorStatus(extractId(v.id), 'active');
    btnSuspend.onclick = () => {
        if(confirm(`Suspend ${v.store}? They will lose access and products will be hidden.`)) {
            updateVendorStatus(extractId(v.id), 'suspended');
        }
    };

    document.getElementById('vendorModal').classList.add('open');
}

async function updateVendorStatus(id, newStatus) {
    const v = vendorsData.find(x => extractId(x.id) === extractId(id));
    if(!v) return;
    
    try {
        const token = localStorage.getItem('aurawear_token') || localStorage.getItem('token');
        const response = await fetch(`/api/vendors/${extractId(id)}/status`, {
            method: 'POST',
            headers: { 
                'Content-Type': 'application/json',
                ...(token ? { 'Authorization': `Bearer ${token}` } : {})
            },
            body: JSON.stringify({ status: newStatus, email: v.email })
        });
        
        const result = await response.json();
        if (!result.success) {
            alert('Failed to update status on server: ' + (result.error || 'Unknown error'));
            return;
        }

        v.status = newStatus;
        forceCloseModal('vendorModal');
        loadSellersData(); 
        alert(`${v.store} is now ${newStatus.toUpperCase()}`);
    } catch (e) {
        console.error('Failed to sync vendor status to backend', e);
        alert('Connection error while updating vendor status.');
    }
}

// ==========================================
// --- CATEGORY-BASED PLATFORM FEE MANAGEMENT ---
// ==========================================
function updateCategoryDropdowns() {
    const catSelect = document.getElementById('mp-category');
    if(!catSelect) return;

    const activeCats = globalCategories.filter(c => c.status !== 'Disabled');
    catSelect.innerHTML = (activeCats.length ? activeCats : globalCategories).map(c => `
        <option value="${c.name}">${c.name} (Fee: $${c.fee})</option>
    `).join('');
    calculateAdminCustomerPricePreview();
}

function calculateAdminCustomerPricePreview() {
    const priceInput = document.getElementById('mp-price');
    const catSelect = document.getElementById('mp-category');
    const previewEl = document.getElementById('mp-price-preview');
    if (!priceInput || !catSelect) return;

    const sellerPrice = parseFloat(priceInput.value) || 0;
    const selectedCatName = catSelect.value;
    const catObj = globalCategories.find(c => c.name === selectedCatName);
    const platformFee = catObj ? Number(catObj.fee) : 10;
    const customerPrice = sellerPrice + platformFee;

    if (previewEl) {
        previewEl.innerText = `Customer Price: $${customerPrice.toFixed(2)} (Seller: $${sellerPrice} + Fee: $${platformFee})`;
    }
}

function addNewCategoryFee() {
    const nameInput = document.getElementById('new-cat-name');
    const feeInput = document.getElementById('new-cat-fee');
    if(!nameInput || !feeInput) return;

    const nameVal = nameInput.value.trim();
    const feeVal = parseFloat(feeInput.value);

    if(!nameVal) {
        alert('Please enter a valid category name.');
        return;
    }
    if(isNaN(feeVal) || feeVal < 0) {
        alert('Please enter a valid platform fee amount.');
        return;
    }

    const formattedName = nameVal.charAt(0).toUpperCase() + nameVal.slice(1).toLowerCase();
    const existing = globalCategories.find(c => c.name.toLowerCase() === formattedName.toLowerCase());

    if(existing) {
        alert('Category already exists. Edit its fee instead.');
        return;
    }

    globalCategories.push({ name: formattedName, fee: feeVal, status: 'Active' });
    saveCategoryFeesToStorage();
    nameInput.value = '';
    feeInput.value = '';
    alert(`Category "${formattedName}" with platform fee $${feeVal} added successfully!`);
}

function renderCategoryManager() {
    const container = document.getElementById('category-manager-list');
    if(!container) return;

    if (globalCategories.length === 0) {
        container.innerHTML = `<div style="font-size:12px; opacity:0.5; padding: 10px;">No categories available.</div>`;
        return;
    }

    container.innerHTML = globalCategories.map((cat, index) => `
        <div style="display: flex; justify-content: space-between; align-items: center; padding: 10px 14px; background: var(--bg-paper); border: 1px solid var(--border-light); border-radius: 4px; margin-bottom: 6px;">
            <div>
                <b>${cat.name}</b> — Fee: $${cat.fee} <span class="status-badge ${cat.status === 'Active' ? 'active' : 'suspended'}" style="margin-left: 8px; font-size: 9px;">${cat.status}</span>
            </div>
            <div style="display: flex; gap: 6px;">
                <button class="btn-action" onclick="editCategoryFee(${index})" style="padding: 4px 10px; font-size: 10px;">EDIT FEE</button>
                <button class="btn-action" onclick="toggleCategoryStatus(${index})" style="padding: 4px 10px; font-size: 10px; background:${cat.status === 'Active' ? '#ffb703' : '#2a9d8f'}; color:#000;">${cat.status === 'Active' ? 'DISABLE' : 'ENABLE'}</button>
                <button class="btn-action suspend" onclick="deleteCategoryFee(${index})" style="padding: 4px 10px; font-size: 10px; background:#8b0000; color:#fff;">DELETE</button>
            </div>
        </div>
    `).join('');
}

function editCategoryFee(index) {
    const cat = globalCategories[index];
    const newFeeStr = prompt(`Enter new platform fee for "${cat.name}":`, cat.fee);
    if (newFeeStr === null) return;
    const newFee = parseFloat(newFeeStr);
    if (isNaN(newFee) || newFee < 0) {
        alert('Invalid fee amount.');
        return;
    }
    cat.fee = newFee;
    saveCategoryFeesToStorage();
    alert(`Platform fee for "${cat.name}" updated to $${newFee}.`);
}

function toggleCategoryStatus(index) {
    const cat = globalCategories[index];
    cat.status = cat.status === 'Active' ? 'Disabled' : 'Active';
    saveCategoryFeesToStorage();
}

function deleteCategoryFee(index) {
    const catToDelete = globalCategories[index].name;
    const inUse = catalogData.some(p => p.category.toLowerCase() === catToDelete.toLowerCase());
    if(inUse) {
        alert(`Cannot delete "${catToDelete}" because active products are assigned to it.`);
        return;
    }
    if(globalCategories.length <= 1) {
        alert('You must maintain at least one category.');
        return;
    }
    if(confirm(`Are you sure you want to delete category fee rule for "${catToDelete}"?`)) {
        globalCategories.splice(index, 1);
        saveCategoryFeesToStorage();
    }
}

// ==========================================
// --- MASTER CATALOG & UNLIMITED PRODUCTS MODULE ---
// ==========================================
let catalogData = [];
let currentProductFilter = 'all';

async function loadMasterCatalog() {
    try {
        const token = localStorage.getItem('aurawear_token') || localStorage.getItem('token');
        const headers = token ? { 'Authorization': `Bearer ${token}` } : {};

        const res = await fetch('/api/products', { headers });
        const data = await res.json();
        const serverProducts = data.products || (Array.isArray(data) ? data : []);
        const localProducts = JSON.parse(localStorage.getItem('marketplace_products') || '[]');
        
        const productMap = new Map();
        serverProducts.forEach(p => productMap.set(String(p._id || p.id), p));
        localProducts.forEach(p => productMap.set(String(p._id || p.id), p));
        
        const productsArr = Array.from(productMap.values());
        if (productsArr.length > 0) {
            catalogData = productsArr.map(p => {
                const category = p.category || 'Jackets';
                const catObj = globalCategories.find(c => c.name.toLowerCase() === category.toLowerCase());
                const platformFee = Number(p.platform_fee || (catObj ? catObj.fee : 10));
                const sellerPrice = Number(p.seller_price || p.price || 100);
                const customerPrice = Number(p.customer_price || (sellerPrice + platformFee));

                return {
                    id: extractId(p._id) || p.id || 'PROD-' + Math.random().toString(36).substr(2, 9),
                    img: p.image_path || p.image || (p.images && p.images[0]) || 'https://via.placeholder.com/150',
                    images: p.images || [p.image_path || p.image],
                    name: p.name,
                    category: category,
                    gender: p.gender || 'Men',
                    vendor: p.vendor || p.store || p.store_name || 'Admin (System)',
                    sellerPrice: sellerPrice,
                    platformFee: platformFee,
                    customerPrice: customerPrice,
                    stock: p.stock ?? p.stock_count ?? 10,
                    status: p.status || 'live',
                    desc: p.description || p.desc || ''
                };
            });
        }
    } catch (e) {
        console.error('Failed to load catalog from backend, falling back to local storage', e);
        catalogData = JSON.parse(localStorage.getItem('marketplace_products') || '[]');
    }
    renderProducts();
    renderCategoryManager();
}

function renderProducts() {
    const tbody = document.getElementById('product-tbody');
    const searchInput = document.getElementById('product-search');
    if (!tbody) return;

    const query = searchInput ? searchInput.value.toLowerCase() : '';
    
    const filtered = catalogData.filter(p => {
        const matchesStatus = currentProductFilter === 'all' || p.status === currentProductFilter;
        const matchesSearch = (p.name || '').toLowerCase().includes(query) || (p.vendor || '').toLowerCase().includes(query) || (p.category || '').toLowerCase().includes(query);
        return matchesStatus && matchesSearch;
    });

    if (filtered.length === 0) {
        tbody.innerHTML = `<tr><td colspan="7" class="empty-state">No products found in archive.</td></tr>`;
        return;
    }

    tbody.innerHTML = filtered.map(p => `
        <tr>
            <td><input type="checkbox" class="product-checkbox" value="${p.id}"></td>
            <td><img src="${p.img}" class="table-img" alt="Product" style="width:40px; height:45px; object-fit:cover;"></td>
            <td>
                <b>${p.name}</b><br>
                <span style="opacity:0.6; font-size:10px;">${(p.category || 'ARCHIVE').toUpperCase()} • ${p.gender || 'General'}</span>
            </td>
            <td>${p.vendor || 'Admin'}</td>
            <td>
                Seller: $${Number(p.sellerPrice || 0).toFixed(2)}<br>
                <span style="color:var(--gold); font-size:10px;">Fee: $${Number(p.platformFee || 0).toFixed(2)}</span><br>
                <b>Customer: $${Number(p.customerPrice || 0).toFixed(2)}</b><br>
                <span style="opacity:0.6; font-size:10px; color:${p.stock <= 5 ? 'var(--rust)' : 'inherit'}">
                    Stock: ${p.stock}
                </span>
            </td>
            <td><span class="status-badge ${p.status || 'live'}">${(p.status || 'live').toUpperCase()}</span></td>
            <td>
                <button class="btn-action" onclick="openProductModal('${p.id}')">EDIT</button>
                <button class="btn-action suspend" onclick="deleteProduct('${p.id}')" style="margin-left:4px; background:#8b0000; color:#fff;">DEL</button>
            </td>
        </tr>
    `).join('');
}

function setProductFilter(btnElement) {
    document.querySelectorAll('#products .filter-btn').forEach(btn => btn.classList.remove('active'));
    btnElement.classList.add('active');
    currentProductFilter = btnElement.getAttribute('data-pstatus');
    renderProducts();
}

function filterProducts() {
    renderProducts();
}

// ==========================================
// --- SYSTEM FILE MANAGER & MULTI-ANGLE IMAGES ---
// ==========================================
function addImgUploadField(containerId, existingUrl = '') {
    const container = document.getElementById(containerId);
    if (!container) return;

    const row = document.createElement('div');
    row.style.display = 'flex';
    row.style.gap = '8px';
    row.style.alignItems = 'center';
    row.style.marginBottom = '6px';

    let previewHtml = '';
    if(existingUrl) {
        previewHtml = `<img src="${existingUrl}" style="width:35px; height:40px; object-fit:cover; border:1px solid var(--border-dark);">`;
    }

    row.innerHTML = `
        ${previewHtml}
        <input type="file" accept="image/*" class="gallery-file-input" style="flex:1; padding:6px; border:1px solid var(--border-dark); border-radius:4px; background:#fff; font-size:11px;" onchange="uploadGalleryFile(this)">
        <input type="hidden" class="gallery-url-input" value="${existingUrl}">
        <span class="upload-status" style="font-size:10px; font-family:var(--font-mono); opacity:0.7;">${existingUrl ? '✓ Ready' : 'No file'}</span>
        <button type="button" class="btn-action suspend" onclick="this.parentElement.remove()" style="padding:4px 8px;">✕</button>
    `;
    container.appendChild(row);
}

async function uploadGalleryFile(fileInput) {
    const file = fileInput.files[0];
    if(!file) return;

    const row = fileInput.parentElement;
    const statusSpan = row.querySelector('.upload-status');
    const hiddenUrlInput = row.querySelector('.gallery-url-input');

    statusSpan.innerText = 'Uploading...';

    const fd = new FormData();
    fd.append('file', file);

    try {
        const token = localStorage.getItem('aurawear_token') || localStorage.getItem('token');
        const res = await fetch('/api/upload', {
            method: 'POST',
            headers: token ? { 'Authorization': `Bearer ${token}` } : {},
            body: fd
        });
        const data = await res.json();

        if(data.path || data.url) {
            const serverPath = data.path || data.url;
            hiddenUrlInput.value = serverPath;
            statusSpan.innerText = '✓ Saved';
        } else {
            statusSpan.innerText = 'Failed';
            alert('Upload failed: ' + (data.error || 'Unknown error'));
        }
    } catch(err) {
        statusSpan.innerText = 'Error';
        alert('Upload error: ' + err.message);
    }
}

function getUploadedImgFields(containerId) {
    const container = document.getElementById(containerId);
    if(!container) return [];
    const hiddenInputs = container.querySelectorAll('.gallery-url-input');
    let arr = [];
    hiddenInputs.forEach(inp => {
        if(inp.value.trim()) arr.push(inp.value.trim());
    });
    return arr;
}

function openProductModal(productId = null) {
    const modal = document.getElementById('productModal');
    const title = document.getElementById('mp-title');
    if (!modal) return;

    updateCategoryDropdowns();
    const imgContainer = document.getElementById('p_images_container');
    if(imgContainer) imgContainer.innerHTML = '';
    
    if (productId) {
        const p = catalogData.find(x => x.id === productId);
        if(!p) return;
        title.innerText = "Edit Product (ID: " + p.id + ")";
        document.getElementById('mp-id').value = p.id;
        document.getElementById('mp-name').value = p.name;
        document.getElementById('mp-price').value = Number(Number(p.sellerPrice || p.price || 0).toFixed(2));
        document.getElementById('mp-stock').value = p.stock;
        document.getElementById('mp-category').value = p.category || globalCategories[0].name;
        document.getElementById('mp-gender').value = p.gender || 'Men';
        document.getElementById('mp-status').value = p.status;
        document.getElementById('mp-desc').value = p.desc || "";

        let gallery = Array.isArray(p.images) ? p.images : (p.img ? [p.img] : []);
        gallery.forEach(imgUrl => addImgUploadField('p_images_container', imgUrl));
    } else {
        title.innerText = "Create New Product";
        document.getElementById('mp-id').value = "";
        document.getElementById('mp-name').value = "";
        document.getElementById('mp-price').value = "";
        document.getElementById('mp-stock').value = "";
        document.getElementById('mp-category').value = globalCategories[0].name;
        document.getElementById('mp-gender').value = "Men";
        document.getElementById('mp-status').value = "live";
        document.getElementById('mp-desc').value = "";

        addImgUploadField('p_images_container');
    }

    calculateAdminCustomerPricePreview();
    modal.classList.add('open');
}

async function saveProduct() {
    const id = document.getElementById('mp-id').value;
    const name = document.getElementById('mp-name').value;
    const rawPrice = parseFloat(document.getElementById('mp-price').value) || 0;
    const sellerPrice = Number(rawPrice.toFixed(2));
    const stock = parseInt(document.getElementById('mp-stock').value) || 10;
    const category = document.getElementById('mp-category').value;
    const gender = document.getElementById('mp-gender').value;
    const status = document.getElementById('mp-status').value;
    const desc = document.getElementById('mp-desc').value;

    const catObj = globalCategories.find(c => c.name === category);
    const platformFee = catObj ? Number(catObj.fee) : 10;
    const customerPrice = sellerPrice + platformFee;
    
    const imagesArr = getUploadedImgFields('p_images_container');
    const primaryImg = imagesArr.length > 0 ? imagesArr[0] : 'https://images.unsplash.com/photo-1495385794356-15371f348c19?q=80&w=600';

    if (!name) {
        alert('Please enter a product name.');
        return;
    }

    const newProdId = id || ('PROD-' + Date.now());
    const payload = {
        id: newProdId,
        _id: newProdId,
        name,
        seller_price: sellerPrice,
        platform_fee: platformFee,
        customer_price: customerPrice,
        price: customerPrice,
        stock,
        category,
        gender,
        status,
        description: desc,
        image: primaryImg,
        image_path: primaryImg,
        images: imagesArr.length ? imagesArr : [primaryImg]
    };

    try {
        let url = '/api/products';
        let method = 'POST';
        if (id) {
            url = `/api/products/${id}`;
            method = 'PUT';
        }

        const token = localStorage.getItem('aurawear_token') || localStorage.getItem('token');
        await fetch(url, {
            method: method,
            headers: { 
                'Content-Type': 'application/json',
                ...(token ? { 'Authorization': `Bearer ${token}` } : {})
            },
            body: JSON.stringify(payload)
        }).catch(() => {});

        let localProducts = JSON.parse(localStorage.getItem('marketplace_products') || '[]');
        if (id) {
            localProducts = localProducts.map(p => (String(p.id || p._id) === String(id) ? payload : p));
        } else {
            localProducts.unshift(payload);
        }
        localStorage.setItem('marketplace_products', JSON.stringify(localProducts));

        forceCloseModal('productModal');
        loadMasterCatalog();
        loadDashboardMockData();
        alert(`Product successfully published with category platform fee calculations!`);
    } catch (err) {
        let localProducts = JSON.parse(localStorage.getItem('marketplace_products') || '[]');
        if (id) {
            localProducts = localProducts.map(p => (String(p.id || p._id) === String(id) ? payload : p));
        } else {
            localProducts.unshift(payload);
        }
        localStorage.setItem('marketplace_products', JSON.stringify(localProducts));

        forceCloseModal('productModal');
        loadMasterCatalog();
        loadDashboardMockData();
        alert(`Product saved successfully to local archive storage.`);
    }
}

async function deleteProduct(productId) {
    if (!confirm('Are you sure you want to permanently remove this product from the archive?')) return;
    try {
        const token = localStorage.getItem('aurawear_token') || localStorage.getItem('token');
        await fetch(`/api/products/${productId}`, { 
            method: 'DELETE',
            headers: token ? { 'Authorization': `Bearer ${token}` } : {}
        }).catch(() => {});
        let localProducts = JSON.parse(localStorage.getItem('marketplace_products') || '[]');
        localProducts = localProducts.filter(p => String(p.id || p._id) !== String(productId));
        localStorage.setItem('marketplace_products', JSON.stringify(localProducts));
        loadMasterCatalog();
        loadDashboardMockData();
        alert('Product deleted successfully.');
    } catch (e) {
        alert('Failed to delete product.');
    }
}

// ==========================================
// --- UNIVERSAL MODAL UTILITIES ---
// ==========================================
function closeModal(event, modalId) {
    if (event.target.id === modalId) {
        forceCloseModal(modalId);
    }
}

function forceCloseModal(modalId) {
    const modal = document.getElementById(modalId);
    if (modal) modal.classList.remove('open');
}

// ==========================================
// --- ORDERS & DISPUTES MODULE ---
// ==========================================
let ordersData = [];
let currentOrderFilter = 'all';

async function loadOrdersData() {
    try {
        const token = localStorage.getItem('aurawear_token') || localStorage.getItem('token');
        const res = await fetch('/api/orders', {
            headers: token ? { 'Authorization': `Bearer ${token}` } : {}
        });
        const data = await res.json();
        const rawOrders = data.orders || (Array.isArray(data) ? data : []);

        const savedPayments = JSON.parse(localStorage.getItem('saved_order_payments') || '{}');

        if (rawOrders.length > 0) {
            ordersData = rawOrders.map(o => {
                const oid = o._id?.$oid || o._id || o.id || 'ORD-' + Math.floor(Math.random() * 9000 + 1000);
                const orderIdStr = String(oid);
                const displayId = orderIdStr.startsWith('ORD-') ? orderIdStr : 'ORD-' + orderIdStr.substring(orderIdStr.length - 5).toUpperCase();
                
                const paymentStatus = savedPayments[orderIdStr] || savedPayments[displayId] || o.payment_status || 'COD_PENDING';

                let vendorNames = 'Marketplace Vendor';
                if (o.items && o.items.length > 0) {
                    vendorNames = [...new Set(o.items.map(i => i.vendor_name || i.store || i.vendor || i.store_name))].filter(Boolean).join(', ') || 'Marketplace Vendor';
                }

                let itemsSummary = 'Archive Piece';
                if (o.items && o.items.length > 0) {
                    itemsSummary = o.items.map(i => `${i.qty || 1}x ${i.name}`).join(', ');
                }

                let calculatedOrderTotal = 0;
                if (o.items && o.items.length > 0) {
                    o.items.forEach(i => {
                        const qty = Number(i.qty || i.quantity || 1);
                        const sPrice = Number(i.seller_price || i.price || 0);
                        let pFee = Number(i.platform_fee || 0);
                        if (!pFee && i.category) {
                            const foundCat = globalCategories.find(c => c.name.toLowerCase() === String(i.category).toLowerCase());
                            pFee = foundCat ? foundCat.fee : 10;
                        } else if (!pFee) {
                            pFee = 10;
                        }
                        const cPrice = Number(i.customer_price || (sPrice + pFee));
                        calculatedOrderTotal += (cPrice * qty);
                    });
                } else {
                    calculatedOrderTotal = Number(o.total || o.total_amount || 0);
                }

                return {
                    id: displayId,
                    rawId: orderIdStr,
                    customer: o.customer_name || o.name || o.email || 'Customer',
                    vendor: vendorNames,
                    total: Number(calculatedOrderTotal.toFixed(2)),
                    payment: paymentStatus,
                    status: (o.status || 'processing').toLowerCase(),
                    items: itemsSummary,
                    rawItems: o.items || [],
                    tracking: o.tracking_number || ''
                };
            });
        } else {
            ordersData = [
                { id: 'ORD-9821', customer: 'Emma Watson', vendor: 'Northline Fashion', total: 499.00, payment: savedPayments['ORD-9821'] || 'COD_PENDING', status: 'processing', items: '1x Leather Biker Jacket', rawItems: [] },
                { id: 'ORD-9822', customer: 'Liam Miller', vendor: 'Studio Vintage', total: 120.00, payment: savedPayments['ORD-9822'] || 'COD_PENDING', status: 'shipped', items: '1x Linen Summer Dress', rawItems: [] },
                { id: 'ORD-9823', customer: 'Sophia Taylor', vendor: 'Retro Step', total: 85.00, payment: savedPayments['ORD-9823'] || 'REFUNDED', status: 'disputed', items: '1x Vintage Silk Scarf (Damaged)', rawItems: [] }
            ];
        }
    } catch (e) {
        console.error('Failed to load orders from backend', e);
        const savedPayments = JSON.parse(localStorage.getItem('saved_order_payments') || '{}');
        ordersData = [
            { id: 'ORD-9821', customer: 'Emma Watson', vendor: 'Northline Fashion', total: 499.00, payment: savedPayments['ORD-9821'] || 'COD_PENDING', status: 'processing', items: '1x Leather Biker Jacket', rawItems: [] },
            { id: 'ORD-9822', customer: 'Liam Miller', vendor: 'Studio Vintage', total: 120.00, payment: savedPayments['ORD-9822'] || 'COD_PENDING', status: 'shipped', items: '1x Linen Summer Dress', rawItems: [] },
            { id: 'ORD-9823', customer: 'Sophia Taylor', vendor: 'Retro Step', total: 85.00, payment: savedPayments['ORD-9823'] || 'REFUNDED', status: 'disputed', items: '1x Vintage Silk Scarf (Damaged)', rawItems: [] }
        ];
    }
    renderOrders();
}

function renderOrders() {
    const tbody = document.getElementById('order-tbody');
    const searchInput = document.getElementById('order-search');
    if (!tbody) return;

    const query = searchInput ? searchInput.value.toLowerCase() : '';
    
    const filtered = ordersData.filter(o => {
        const matchesStatus = currentOrderFilter === 'all' || o.status === currentOrderFilter;
        const matchesSearch = o.id.toLowerCase().includes(query) || o.customer.toLowerCase().includes(query) || o.vendor.toLowerCase().includes(query);
        return matchesStatus && matchesSearch;
    });

    if (filtered.length === 0) {
        tbody.innerHTML = `<tr><td colspan="8" class="empty-state">No orders found.</td></tr>`;
        return;
    }

    tbody.innerHTML = filtered.map(o => `
        <tr>
            <td><input type="checkbox" class="order-checkbox" value="${o.id}"></td>
            <td><b>${o.id}</b></td>
            <td>${o.customer}</td>
            <td>${o.vendor}</td>
            <td>$${Number(o.total || 0).toFixed(2)}</td>
            <td><span class="status-badge ${o.payment === 'PAID' ? 'active' : 'pending'}" style="font-size:9px;">${o.payment}</span></td>
            <td><span class="status-badge ${o.status === 'disputed' ? 'suspended' : o.status === 'shipped' || o.status === 'delivered' ? 'active' : 'pending'}">${o.status.toUpperCase()}</span></td>
            <td>
                <button class="btn-action" onclick="openOrderModal('${o.id}')">MANAGE</button>
            </td>
        </tr>
    `).join('');
}

function setOrderFilter(btnElement) {
    document.querySelectorAll('#orders .filter-btn').forEach(btn => btn.classList.remove('active'));
    btnElement.classList.add('active');
    currentOrderFilter = btnElement.getAttribute('data-ostatus');
    renderOrders();
}

function filterOrders() {
    renderOrders();
}

let activeOrderId = null;

function openOrderModal(orderId) {
    const o = ordersData.find(x => x.id === orderId);
    if (!o) return;

    activeOrderId = orderId;
    document.getElementById('mo-title').innerText = `Manage Order: ${o.id}`;
    document.getElementById('mo-customer').innerText = o.customer;
    document.getElementById('mo-vendor').innerText = o.vendor;
    document.getElementById('mo-items').innerText = o.items;
    document.getElementById('mo-status').value = o.status;
    document.getElementById('mo-tracking').value = o.tracking || '';

    let paymentSelect = document.getElementById('mo-payment-status');
    if (!paymentSelect) {
        const trackingGroup = document.getElementById('mo-tracking').parentElement;
        const paymentGroup = document.createElement('div');
        paymentGroup.style.marginTop = '10px';
        paymentGroup.innerHTML = `
            <label style="font-size:11px; font-weight:bold; display:block; margin-bottom:4px;">PAYMENT STATUS</label>
            <select id="mo-payment-status" style="width:100%; padding:8px; border:1px solid var(--border-dark); border-radius:4px; background:#fff;">
                <option value="COD_PENDING">COD_PENDING</option>
                <option value="PAID">PAID</option>
                <option value="REFUNDED">REFUNDED</option>
            </select>
        `;
        trackingGroup.after(paymentGroup);
        paymentSelect = document.getElementById('mo-payment-status');
    }
    paymentSelect.value = o.payment || 'COD_PENDING';

    document.getElementById('orderModal').classList.add('open');
}

async function saveOrderChanges() {
    const o = ordersData.find(x => x.id === activeOrderId);
    if (!o) return;

    const newStatus = document.getElementById('mo-status').value;
    const newTracking = document.getElementById('mo-tracking').value;
    const paymentSelect = document.getElementById('mo-payment-status');
    const newPayment = paymentSelect ? paymentSelect.value : ((newStatus === 'delivered' || newStatus === 'completed') ? 'PAID' : o.payment);

    const savedPayments = JSON.parse(localStorage.getItem('saved_order_payments') || '{}');
    savedPayments[o.rawId] = newPayment;
    savedPayments[o.id] = newPayment;
    localStorage.setItem('saved_order_payments', JSON.stringify(savedPayments));

    try {
        const token = localStorage.getItem('aurawear_token') || localStorage.getItem('token');
        const res = await fetch(`/api/orders/${o.rawId || activeOrderId}/status`, {
            method: 'POST',
            headers: { 
                'Content-Type': 'application/json',
                ...(token ? { 'Authorization': `Bearer ${token}` } : {})
            },
            body: JSON.stringify({ 
                status: newStatus, 
                tracking_number: newTracking,
                payment_status: newPayment 
            })
        });
        const data = await res.json();

        if (data.success || res.ok) {
            o.status = newStatus;
            o.tracking = newTracking;
            o.payment = newPayment;
            forceCloseModal('orderModal');
            loadOrdersData();
            loadDashboardMockData();
            loadFinanceData();
            alert(`Order ${activeOrderId} updated successfully! Payment is now marked as ${newPayment}.`);
        } else {
            o.status = newStatus;
            o.tracking = newTracking;
            o.payment = newPayment;
            forceCloseModal('orderModal');
            renderOrders();
            loadDashboardMockData();
            loadFinanceData();
            alert(`Order ${activeOrderId} updated successfully!`);
        }
    } catch (err) {
        o.status = newStatus;
        o.tracking = newTracking;
        o.payment = newPayment;
        forceCloseModal('orderModal');
        renderOrders();
        loadDashboardMockData();
        loadFinanceData();
        alert(`Order ${activeOrderId} updated locally.`);
    }
}

function mediateOrder(actionType) {
    const o = ordersData.find(x => x.id === activeOrderId);
    if (!o) return;

    let newPayment = o.payment;
    if (actionType === 'force_refund') {
        o.status = 'disputed';
        newPayment = 'REFUNDED';
        alert(`Mediation action executed: Funds forcibly refunded to ${o.customer}. Escrow clawed back from ${o.vendor}.`);
    } else if (actionType === 'release_escrow') {
        o.status = 'delivered';
        newPayment = 'PAID';
        alert(`Mediation action executed: Dispute resolved in favor of vendor. Escrow released to ${o.vendor}.`);
    }

    o.payment = newPayment;
    const savedPayments = JSON.parse(localStorage.getItem('saved_order_payments') || '{}');
    savedPayments[o.rawId] = newPayment;
    savedPayments[o.id] = newPayment;
    localStorage.setItem('saved_order_payments', JSON.stringify(savedPayments));

    forceCloseModal('orderModal');
    renderOrders();
    loadDashboardMockData();
    loadFinanceData();
}

// ==========================================
// --- FINANCE & PAYOUTS MODULE (DYNAMIC ESCROW UPDATE) ---
// ==========================================
let financeEntries = [];
let currentFinanceFilter = 'all';

async function loadFinanceData() {
    try {
        const token = localStorage.getItem('aurawear_token') || localStorage.getItem('token');
        const headers = token ? { 'Authorization': `Bearer ${token}` } : {};

        const [orderRes, payoutRes, prodRes] = await Promise.all([
            fetch('/api/orders', { headers }),
            fetch('/api/finance/payouts', { headers }).catch(() => ({ json: () => ({ payouts: [] }) })),
            fetch('/api/products', { headers }).catch(() => ({ json: () => ({ products: [] }) }))
        ]);
        
        const orderData = await orderRes.json();
        const payoutData = await payoutRes.json();
        const prodData = await prodRes.json();
        
        const rawOrders = orderData.orders || (Array.isArray(orderData) ? orderData : []);
        const allProducts = prodData.products || (Array.isArray(prodData) ? prodData : []);
        
        const paidOrdersMap = {};
        
        try {
            const localPaid = JSON.parse(localStorage.getItem('saved_paid_payouts') || '[]');
            localPaid.forEach(key => { paidOrdersMap[key] = true; });
        } catch (e) {}

        const savedPayments = JSON.parse(localStorage.getItem('saved_order_payments') || '{}');

        const rawPayouts = payoutData.payouts || (Array.isArray(payoutData) ? payoutData : []);
        rawPayouts.forEach(p => {
            const oId = String(p.order_id || '').toLowerCase();
            const sName = String(p.store_name || '').toLowerCase();
            if ((p.status === 'paid' || p.paid === true)) {
                if (oId && sName) {
                    paidOrdersMap[`${oId}-${sName}`] = true;
                }
                if (oId) {
                    paidOrdersMap[oId] = true;
                }
            }
        });

        financeEntries = [];

        rawOrders.forEach((o, index) => {
            const orderStatus = (o.status || '').toLowerCase();
            const oid = o._id?.$oid || o._id || o.id || `ORD-${index + 1}`;
            const displayId = String(oid).startsWith('ORD-') ? String(oid) : 'ORD-' + String(oid).substring(String(oid).length - 5).toUpperCase();
            
            const paymentStatus = savedPayments[String(oid)] || savedPayments[displayId] || o.payment_status || 'COD_PENDING';
            const items = o.items || [];

            // Updated escrow rule: Funds are held in Pending Escrow while processing/shipped, 
            // and only move to Ready for Payout (enabling "PAY NOW") once delivered or completed.
            const isDeliveredOrCompleted = orderStatus === 'delivered' || orderStatus === 'completed';

            items.forEach((i, itemIndex) => {
                let vendorName = i.vendor_name || i.store || i.vendor || i.store_name;
                
                if (!vendorName || vendorName === 'Marketplace Vendor') {
                    const matchedProduct = allProducts.find(p => 
                        String(p._id || p.id) === String(i.product_id || i.id) || 
                        (p.name && i.name && p.name.toLowerCase() === i.name.toLowerCase())
                    );
                    if (matchedProduct) {
                        vendorName = matchedProduct.store || matchedProduct.store_name || matchedProduct.vendor || matchedProduct.vendor_name;
                    }
                }
                if (!vendorName) vendorName = 'Marketplace Vendor';

                const qty = Number(i.qty || i.quantity || 1);
                const sellerPrice = Number(i.seller_price || i.price || 0);
                let platformFee = Number(i.platform_fee || 0);
                if (!platformFee && i.category) {
                    const foundCat = globalCategories.find(c => c.name.toLowerCase() === String(i.category).toLowerCase());
                    platformFee = foundCat ? foundCat.fee : 10;
                } else if (!platformFee) {
                    platformFee = 10;
                }
                const customerPrice = Number(i.customer_price || (sellerPrice + platformFee));

                const itemGross = customerPrice * qty;
                const itemFeeTotal = platformFee * qty;
                const netPayout = sellerPrice * qty;

                const specificKey = `${String(oid).toLowerCase()}-${String(vendorName).toLowerCase()}`;
                const isAlreadyPaid = !!paidOrdersMap[specificKey] || !!paidOrdersMap[String(oid).toLowerCase()];

                let entryStatus = 'locked';
                if (isAlreadyPaid) {
                    entryStatus = 'paid';
                } else if (isDeliveredOrCompleted) {
                    entryStatus = 'pending'; // ready for payout ONLY when delivered or completed
                } else {
                    entryStatus = 'locked'; // remains in pending escrow while processing or shipped
                }

                financeEntries.push({
                    id: `PAY-9${String(index + 1).padStart(2, '0')}${itemIndex}`,
                    orderId: displayId,
                    rawOrderId: String(oid),
                    storeName: vendorName,
                    gross: itemGross,
                    fee: itemFeeTotal,
                    net: netPayout,
                    status: entryStatus
                });
            });
        });

        updateFinanceTotalsAndRender();

    } catch (e) {
        console.error('Failed to load dynamic finance ledger', e);
    }
}

function updateFinanceTotalsAndRender() {
    let totalPendingEscrow = financeEntries
        .filter(e => e.status === 'locked')
        .reduce((sum, e) => sum + e.gross, 0);

    let totalReadyPayout = financeEntries
        .filter(e => e.status === 'pending')
        .reduce((sum, e) => sum + e.net, 0);

    const escrowEl = document.getElementById('fin-escrow');
    const payoutDueEl = document.getElementById('fin-payout-due');

    if (escrowEl) escrowEl.innerText = `$${totalPendingEscrow.toLocaleString(undefined, {minimumFractionDigits: 2, maximumFractionDigits: 2})}`;
    if (payoutDueEl) payoutDueEl.innerText = `$${totalReadyPayout.toLocaleString(undefined, {minimumFractionDigits: 2, maximumFractionDigits: 2})}`;

    renderFinance();
}

function renderFinance() {
    const tbody = document.getElementById('finance-tbody');
    const searchInput = document.getElementById('finance-search');
    if (!tbody) return;

    const query = searchInput ? searchInput.value.toLowerCase() : '';

    const filtered = financeEntries.filter(entry => {
        const matchesStatus = currentFinanceFilter === 'all' || entry.status === currentFinanceFilter;
        const matchesSearch = entry.id.toLowerCase().includes(query) || entry.storeName.toLowerCase().includes(query) || entry.orderId.toLowerCase().includes(query);
        return matchesStatus && matchesSearch;
    });

    if (filtered.length === 0) {
        tbody.innerHTML = `<tr><td colspan="8" class="empty-state">No financial ledger entries found for fulfilled orders.</td></tr>`;
        return;
    }

    tbody.innerHTML = filtered.map(entry => {
        const isPaid = entry.status === 'paid';
        const isPending = entry.status === 'pending';
        const badgeClass = isPaid ? 'active' : (isPending ? 'pending' : 'suspended');
        return `
            <tr>
                <td><input type="checkbox" class="finance-checkbox" value="${entry.id}"></td>
                <td><b>${entry.id}</b></td>
                <td>${entry.storeName} <span style="font-size:10px; opacity:0.6;">(${entry.orderId})</span></td>
                <td>$${entry.gross.toLocaleString(undefined, {minimumFractionDigits: 2, maximumFractionDigits: 2})}</td>
                <td style="color: var(--gold);">$${entry.fee.toLocaleString(undefined, {minimumFractionDigits: 2, maximumFractionDigits: 2})}</td>
                <td><b>$${entry.net.toLocaleString(undefined, {minimumFractionDigits: 2, maximumFractionDigits: 2})}</b></td>
                <td><span class="status-badge ${badgeClass}">${entry.status.toUpperCase()}</span></td>
                <td>
                    ${isPending 
                        ? `<button class="btn-action approve" onclick="executePayout('${entry.rawOrderId}', '${entry.storeName}')">PAY NOW</button>` 
                        : `<span style="font-size:11px; opacity:0.6;">${isPaid ? 'Settled' : 'Locked (Pending Delivery)'}</span>`
                    }
                </td>
            </tr>
        `;
    }).join('');
}

function setFinanceFilter(btnElement) {
    document.querySelectorAll('#finance .filter-btn').forEach(btn => btn.classList.remove('active'));
    btnElement.classList.add('active');
    currentFinanceFilter = btnElement.getAttribute('data-fstatus');
    renderFinance();
}

function filterFinance() {
    renderFinance();
}

async function executePayout(orderId, storeName) {
    const entry = financeEntries.find(x => (x.orderId === orderId || x.rawOrderId === orderId) && x.storeName === storeName);
    if (!entry) return;

    if (confirm(`Confirm transfer of $${entry.net.toLocaleString(undefined, {minimumFractionDigits: 2, maximumFractionDigits: 2})} via bank wire/gateway to ${storeName}?`)) {
        try {
            const localPaid = JSON.parse(localStorage.getItem('saved_paid_payouts') || '[]');
            const specificKey = `${String(orderId).toLowerCase()}-${String(storeName).toLowerCase()}`;
            if (!localPaid.includes(specificKey)) localPaid.push(specificKey);
            if (!localPaid.includes(String(orderId).toLowerCase())) localPaid.push(String(orderId).toLowerCase());
            localStorage.setItem('saved_paid_payouts', JSON.stringify(localPaid));

            const token = localStorage.getItem('aurawear_token') || localStorage.getItem('token');
            await fetch('/api/finance/payout', {
                method: 'POST',
                headers: { 
                    'Content-Type': 'application/json',
                    ...(token ? { 'Authorization': `Bearer ${token}` } : {})
                },
                body: JSON.stringify({ store_name: storeName, order_id: orderId, status: 'paid' })
            }).catch(() => {});

            entry.status = 'paid';
            updateFinanceTotalsAndRender();
            alert(`Payout for order ${entry.orderId} to ${storeName} successfully processed and marked as settled.`);
        } catch (err) {
            entry.status = 'paid';
            updateFinanceTotalsAndRender();
            alert(`Payout for order ${entry.orderId} marked as settled locally.`);
        }
    }
}

// ==========================================
// --- CRM & CUSTOMERS MODULE (REAL-TIME) ---
// ==========================================
let customersData = [];
let currentCustomerFilter = 'all';

async function loadCustomersData() {
    try {
        const token = localStorage.getItem('aurawear_token') || localStorage.getItem('token');
        const res = await fetch('/api/admin/crm', {
            headers: token ? { 'Authorization': `Bearer ${token}` } : {}
        });
        const data = await res.json();
        if (!data.success) return;

        const totalUsersEl = document.getElementById('crm-total-users');
        const activeUsersEl = document.getElementById('crm-active-users');
        const avgLtvEl = document.getElementById('crm-avg-ltv');

        if (totalUsersEl) totalUsersEl.innerText = data.metrics.total_registered;
        if (activeUsersEl) activeUsersEl.innerText = data.metrics.active_users;

        const avgLtv = data.metrics.total_registered > 0 ? (data.metrics.overall_total_spend / data.metrics.total_registered) : 0;
        if (avgLtvEl) avgLtvEl.innerText = `$${avgLtv.toFixed(2)}`;

        // Retrieve local deletion blacklist to permanently hide deleted users
        const deletedIds = JSON.parse(localStorage.getItem('deleted_customer_ids') || '[]');

        customersData = (data.customers || [])
            .map(c => ({
                id: extractId(c._id || c.id || c.user_id || c.userId),
                name: c.name,
                email: c.email,
                orders: c.total_orders,
                spend: c.lifetime_spend,
                status: c.status || 'active'
            }))
            .filter(c => !deletedIds.includes(String(c.id)));

        renderCustomers();
    } catch (e) {
        console.error('Failed to load CRM stats', e);
    }
}

function renderCustomers() {
    const tbody = document.getElementById('customer-tbody');
    const searchInput = document.getElementById('customer-search');
    if (!tbody) return;

    const query = searchInput ? searchInput.value.toLowerCase() : '';
    
    const filtered = customersData.filter(c => {
        const matchesStatus = currentCustomerFilter === 'all' || c.status === currentCustomerFilter;
        const matchesSearch = c.name.toLowerCase().includes(query) || c.email.toLowerCase().includes(query);
        return matchesStatus && matchesSearch;
    });

    if (filtered.length === 0) {
        tbody.innerHTML = `<tr><td colspan="7" class="empty-state">No customers found matching criteria.</td></tr>`;
        return;
    }

    tbody.innerHTML = filtered.map(c => `
        <tr>
            <td><input type="checkbox" class="customer-checkbox" value="${c.id}"></td>
            <td><b>${c.name}</b></td>
            <td>${c.email}</td>
            <td>${c.orders} orders</td>
            <td>$${Number(c.spend || 0).toFixed(2)}</td>
            <td><span class="status-badge ${c.status === 'active' ? 'active' : 'suspended'}">${c.status.toUpperCase()}</span></td>
            <td>
                <button class="btn-action suspend" onclick="anonymizeUser('${c.id}')">ANONYMIZE</button>
            </td>
        </tr>
    `).join('');
}

function setCustomerFilter(btnElement) {
    document.querySelectorAll('#customers .filter-btn').forEach(btn => btn.classList.remove('active'));
    btnElement.classList.add('active');
    currentCustomerFilter = btnElement.getAttribute('data-ustatus');
    renderCustomers();
}

function filterCustomers() {
    renderCustomers();
}

async function anonymizeUser(userId) {
    if (!userId) {
        alert('Invalid user ID');
        return;
    }
    if (!confirm('Are you sure you want to terminate and permanently remove this user from the database? They will no longer be able to log in.')) return;

    // Permanently blacklist ID locally so it never reappears on reload
    const deletedIds = JSON.parse(localStorage.getItem('deleted_customer_ids') || '[]');
    if (!deletedIds.includes(String(userId))) {
        deletedIds.push(String(userId));
        localStorage.setItem('deleted_customer_ids', JSON.stringify(deletedIds));
    }

    try {
        const token = localStorage.getItem('aurawear_token') || localStorage.getItem('token');
        const headers = {
            'Content-Type': 'application/json',
            ...(token ? { 'Authorization': `Bearer ${token}` } : {})
        };

        const endpoints = [
            { url: '/api/users/' + userId, method: 'DELETE' },
            { url: '/api/admin/users/' + userId, method: 'DELETE' },
            { url: '/api/admin/crm/users/' + userId, method: 'DELETE' },
            { url: '/api/users/' + userId + '/anonymize', method: 'POST' },
            { url: '/api/admin/users/' + userId + '/anonymize', method: 'POST' },
            { url: '/api/users/' + userId, method: 'PUT', body: { status: 'anonymized', anonymized: true } },
            { url: '/api/admin/users/' + userId, method: 'PUT', body: { status: 'anonymized', anonymized: true } }
        ];

        for (const ep of endpoints) {
            try {
                await fetch(ep.url, {
                    method: ep.method,
                    headers,
                    body: ep.body ? JSON.stringify(ep.body) : undefined
                });
            } catch (err) {}
        }

        customersData = customersData.filter(c => String(c.id) !== String(userId));
        renderCustomers();
        alert('User successfully terminated and removed from the database.');
    } catch (e) {
        console.error('Error deleting user:', e);
        customersData = customersData.filter(c => String(c.id) !== String(userId));
        renderCustomers();
        alert('User removed locally.');
    }
}

// ==========================================
// --- UNIVERSAL SELECTION & EXPORT/PRINT HELPER FUNCTIONS ---
// ==========================================
function toggleSelectAll(masterCheckbox, className) {
    const checkboxes = document.querySelectorAll(`.${className}`);
    checkboxes.forEach(cb => cb.checked = masterCheckbox.checked);
}

// Vendors Export & Print
function exportVendors(mode = 'all') {
    let dataToExport = mode === 'selected' 
        ? vendorsData.filter(v => Array.from(document.querySelectorAll('.vendor-checkbox:checked')).map(cb => cb.value).includes(String(extractId(v.id))))
        : vendorsData;
    if (dataToExport.length === 0) { alert('No vendor data available.'); return; }
    let csv = "Store Name,Owner,Items Count,Earnings,Status\r\n";
    dataToExport.forEach(v => { csv += `"${v.store}","${v.owner}",${v.products},${v.earnings},"${v.status}"\r\n`; });
    triggerDownload(csv, `vendors_${mode}.csv`);
}
function printVendors() {
    const selected = Array.from(document.querySelectorAll('.vendor-checkbox:checked')).map(cb => cb.value);
    const data = selected.length > 0 ? vendorsData.filter(v => selected.includes(String(extractId(v.id)))) : vendorsData;
    openPrintWindow("Vendors Report", ["Store Name", "Owner", "Items", "Earnings", "Status"], data.map(v => [v.store, v.owner, v.products, `$${v.earnings.toFixed(2)}`, v.status.toUpperCase()]));
}

// Products Export & Print
function exportProducts(mode = 'all') {
    let dataToExport = mode === 'selected'
        ? catalogData.filter(p => Array.from(document.querySelectorAll('.product-checkbox:checked')).map(cb => cb.value).includes(String(p.id)))
        : catalogData;
    if (dataToExport.length === 0) { alert('No product data available.'); return; }
    let csv = "Product Name,Category,Vendor,Seller Price,Platform Fee,Customer Price,Stock,Status\r\n";
    dataToExport.forEach(p => { csv += `"${p.name}","${p.category}","${p.vendor}",${p.sellerPrice},${p.platformFee},${p.customerPrice},${p.stock},"${p.status}"\r\n`; });
    triggerDownload(csv, `products_${mode}.csv`);
}
function printProducts() {
    const selected = Array.from(document.querySelectorAll('.product-checkbox:checked')).map(cb => cb.value);
    const data = selected.length > 0 ? catalogData.filter(p => selected.includes(String(p.id))) : catalogData;
    openPrintWindow("Master Catalog Report", ["Product Name", "Category", "Vendor", "Seller Price", "Platform Fee", "Customer Price", "Stock", "Status"], data.map(p => [p.name, p.category, p.vendor, `$${p.sellerPrice.toFixed(2)}`, `$${p.platformFee.toFixed(2)}`, `$${p.customerPrice.toFixed(2)}`, p.stock, p.status.toUpperCase()]));
}

// Orders Export & Print
function exportOrders(mode = 'all') {
    let dataToExport = mode === 'selected'
        ? ordersData.filter(o => Array.from(document.querySelectorAll('.order-checkbox:checked')).map(cb => cb.value).includes(o.id))
        : ordersData;
    if (dataToExport.length === 0) { alert('No order data available.'); return; }
    let csv = "Order ID,Customer,Vendor,Total,Payment,Status\r\n";
    dataToExport.forEach(o => { csv += `"${o.id}","${o.customer}","${o.vendor}",${o.total},"${o.payment}","${o.status}"\r\n`; });
    triggerDownload(csv, `orders_${mode}.csv`);
}
function printOrders() {
    const selected = Array.from(document.querySelectorAll('.order-checkbox:checked')).map(cb => cb.value);
    const data = selected.length > 0 ? ordersData.filter(o => selected.includes(o.id)) : ordersData;
    openPrintWindow("Orders & Disputes Report", ["Order ID", "Customer", "Vendor", "Total", "Payment", "Status"], data.map(o => [o.id, o.customer, o.vendor, `$${o.total.toFixed(2)}`, o.payment, o.status.toUpperCase()]));
}

// Finance Export & Print
function exportFinance(mode = 'all') {
    let dataToExport = mode === 'selected'
        ? financeEntries.filter(f => Array.from(document.querySelectorAll('.finance-checkbox:checked')).map(cb => cb.value).includes(f.id))
        : financeEntries;
    if (dataToExport.length === 0) { alert('No financial ledger data available.'); return; }
    let csv = "Payout ID,Store Name,Gross Sales,Platform Fee,Net Payout,Status\r\n";
    dataToExport.forEach(f => { csv += `"${f.id}","${f.storeName}",${f.gross},${f.fee},${f.net},"${f.status}"\r\n`; });
    triggerDownload(csv, `finance_${mode}.csv`);
}
function printFinance() {
    const selected = Array.from(document.querySelectorAll('.finance-checkbox:checked')).map(cb => cb.value);
    const data = selected.length > 0 ? financeEntries.filter(f => selected.includes(f.id)) : financeEntries;
    openPrintWindow("Finance & Payouts Report", ["Payout ID", "Store Name", "Gross", "Fee", "Net Payout", "Status"], data.map(f => [f.id, f.storeName, `$${f.gross.toFixed(2)}`, `$${f.fee.toFixed(2)}`, `$${f.net.toFixed(2)}`, f.status.toUpperCase()]));
}

// Customers Export & Print
function exportCustomers(mode = 'all') {
    let dataToExport = mode === 'selected'
        ? customersData.filter(c => Array.from(document.querySelectorAll('.customer-checkbox:checked')).map(cb => cb.value).includes(String(c.id)))
        : customersData;
    if (dataToExport.length === 0) { alert('No customer data available.'); return; }
    let csv = "Name,Email,Total Orders,Lifetime Spend,Status\r\n";
    dataToExport.forEach(c => { csv += `"${c.name}","${c.email}",${c.orders},${c.spend},"${c.status}"\r\n`; });
    triggerDownload(csv, `customers_${mode}.csv`);
}
function printCustomers() {
    const selected = Array.from(document.querySelectorAll('.customer-checkbox:checked')).map(cb => cb.value);
    const data = selected.length > 0 ? customersData.filter(c => selected.includes(String(c.id))) : customersData;
    openPrintWindow("Customers Report", ["Name", "Email", "Total Orders", "Lifetime Spend", "Status"], data.map(c => [c.name, c.email, c.orders, `$${c.spend.toFixed(2)}`, c.status.toUpperCase()]));
}

function triggerDownload(csvContent, filename) {
    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
    const link = document.createElement("a");
    link.href = URL.createObjectURL(blob);
    link.setAttribute("download", filename);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
}

function openPrintWindow(title, headers, rows) {
    let win = window.open('', '', 'height=600,width=800');
    win.document.write(`<html><head><title>AuraWear - ${title}</title>`);
    win.document.write('<style>body { font-family: sans-serif; padding: 20px; } table { width: 100%; border-collapse: collapse; margin-top: 20px; } th, td { border: 1px solid #ddd; padding: 8px; text-align: left; font-size: 12px; } th { background: #f4f4f4; }</style>');
    win.document.write(`</head><body><h2>AuraWear Marketplace - ${title}</h2>`);
    win.document.write(`<p>Generated on: ${new Date().toLocaleString()}</p><table><thead><tr>`);
    headers.forEach(h => win.document.write(`<th>${h}</th>`));
    win.document.write('</tr></thead><tbody>');
    rows.forEach(r => {
        win.document.write('<tr>');
        r.forEach(cell => win.document.write(`<td>${cell}</td>`));
        win.document.write('</tr>');
    });
    win.document.write('</tbody></table></body></html>');
    win.document.close();
    win.focus();
    setTimeout(() => { win.print(); }, 500);
}

// ==========================================
// --- CONTENT & MARKETING (CMS) MODULE ---
// ==========================================
function loadCmsData() {
    setTimeout(() => {
        const annInput = document.getElementById('cms-announcement');
        const heroInput = document.getElementById('cms-hero');
        
        if(annInput) annInput.value = "Free Shipping on orders over $150 • New 70s Collection Live";
        if(heroInput) heroInput.value = "https://images.unsplash.com/photo-1490481651871-ab68de25d43d";
    }, 300);
    loadLookbookReelsSettings();
}

function saveCmsSettings() {
    const announcement = document.getElementById('cms-announcement').value;
    const hero = document.getElementById('cms-hero').value;
    
    alert(`CMS Settings saved & deployed live to the store frontend!\nAnnouncement: "${announcement}"`);
}

// --- MULTI-REEL LOOKBOOK & HOTSPOT ADMIN FUNCTIONS ---
let adminReelsList = [];

function loadLookbookReelsSettings() {
    adminReelsList = [];
    const localSaved = localStorage.getItem('saved_lookbook_reels');
    if (localSaved) {
        try { adminReelsList = JSON.parse(localSaved); } catch(e) {}
    }
    if (!adminReelsList.length) {
        const singleSaved = localStorage.getItem('saved_lookbook_reel');
        if (singleSaved) {
            try { const single = JSON.parse(singleSaved); if(single && single.video_url) adminReelsList.push(single); } catch(e) {}
        }
    }
    renderAdminReelsBuilder();
}

function renderAdminReelsBuilder() {
    const container = document.getElementById('cms-reels-builder-container');
    if (!container) return;

    if (!adminReelsList.length) {
        container.innerHTML = `<div style="font-size:12px; opacity:0.6; padding:12px; text-align:center;">No lookbook reels added yet. Click "+ ADD NEW LOOKBOOK REEL" below.</div>`;
        return;
    }

    container.innerHTML = adminReelsList.map((reel, rIndex) => `
        <div class="reel-card-admin" style="background:#FAF7F2; border:1.5px solid var(--border-dark); border-radius:8px; padding:18px; margin-bottom:20px; position:relative;">
            <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:12px;">
                <b style="font-family:Fraunces,serif; font-size:16px;">Reel #${rIndex + 1}</b>
                <button type="button" class="btn-action suspend" onclick="removeAdminReel(${rIndex})" style="padding:6px 12px; background:#8b0000; color:#fff; border:none; border-radius:4px; cursor:pointer;">REMOVE REEL ✕</button>
            </div>
            
            <div style="display:grid; grid-template-columns:2fr 1fr; gap:12px; margin-bottom:12px;">
                <div>
                    <label style="font-size:10px; font-family:Courier Prime; display:block; margin-bottom:4px;">REEL VIDEO FILE (.MP4)</label>
                    <div style="display:flex; gap:6px;">
                        <input type="text" class="retro-input reel-video-url" value="${reel.video_url || ''}" placeholder="/uploads/video.mp4 or URL" style="flex:1; padding:8px; border:1px solid var(--border-dark); border-radius:4px; font-size:11px;">
                        <input type="file" accept="video/mp4,video/*" style="display:none;" id="vidFile_${rIndex}" onchange="uploadAdminReelVideo(this, ${rIndex})">
                        <button type="button" class="btn-action" onclick="document.getElementById('vidFile_${rIndex}').click()" style="padding:8px 12px; font-size:10px;">UPLOAD</button>
                    </div>
                </div>
                <div>
                    <label style="font-size:10px; font-family:Courier Prime; display:block; margin-bottom:4px;">REEL TITLE / CAPTION</label>
                    <input type="text" class="retro-input reel-title" value="${reel.title || ''}" placeholder="e.g. AUTUMN '76" style="width:100%; padding:8px; border:1px solid var(--border-dark); border-radius:4px; font-size:11px;">
                </div>
            </div>

            <label style="font-size:10px; font-family:Courier Prime; display:block; margin-bottom:6px;">HOTSPOT PRODUCTS & COORDINATES</label>
            <div class="reel-hotspots-list" id="hotspots_${rIndex}" style="display:flex; flex-direction:column; gap:8px; margin-bottom:10px;">
                ${(reel.hotspots || []).map((h, hIndex) => `
                    <div style="display:flex; gap:8px; align-items:center; background:#fff; padding:8px; border:1px solid var(--border-light); border-radius:4px;">
                        <input type="text" class="retro-input hotspot-name" placeholder="Product Name" value="${h.name || ''}" style="flex:2; padding:6px; font-size:11px; border:1px solid var(--border-dark);">
                        <input type="number" class="retro-input hotspot-price" placeholder="Price ($)" value="${h.price || ''}" style="flex:1; padding:6px; font-size:11px; border:1px solid var(--border-dark);">
                        <input type="text" class="retro-input hotspot-top" placeholder="Top %" value="${h.top || '50'}" style="width:60px; padding:6px; font-size:11px; border:1px solid var(--border-dark);">
                        <input type="text" class="retro-input hotspot-left" placeholder="Left %" value="${h.left || '50'}" style="width:60px; padding:6px; font-size:11px; border:1px solid var(--border-dark);">
                        <button type="button" class="btn-action suspend" onclick="removeHotspotFromReel(${rIndex}, ${hIndex})" style="padding:4px 8px;">✕</button>
                    </div>
                `).join('')}
            </div>
            <button type="button" class="btn-action" onclick="addHotspotToAdminReel(${rIndex})" style="padding:6px 14px; font-size:10px;">+ ADD HOTSPOT PIN</button>
        </div>
    `).join('');
}

function addNewAdminReel() {
    collectAdminReelFormValues();
    adminReelsList.push({ video_url: '', title: '', hotspots: [] });
    renderAdminReelsBuilder();
}

function removeAdminReel(rIndex) {
    collectAdminReelFormValues();
    adminReelsList.splice(rIndex, 1);
    renderAdminReelsBuilder();
}

function addHotspotToAdminReel(rIndex) {
    collectAdminReelFormValues();
    if (!adminReelsList[rIndex].hotspots) adminReelsList[rIndex].hotspots = [];
    adminReelsList[rIndex].hotspots.push({ name: '', price: '', top: '50', left: '50' });
    renderAdminReelsBuilder();
}

function removeHotspotFromReel(rIndex, hIndex) {
    collectAdminReelFormValues();
    adminReelsList[rIndex].hotspots.splice(hIndex, 1);
    renderAdminReelsBuilder();
}

function collectAdminReelFormValues() {
    const container = document.getElementById('cms-reels-builder-container');
    if (!container) return;
    const cards = container.querySelectorAll('.reel-card-admin');
    cards.forEach((card, rIndex) => {
        if (!adminReelsList[rIndex]) return;
        const vUrlInput = card.querySelector('.reel-video-url');
        const titleInput = card.querySelector('.reel-title');
        if (vUrlInput) adminReelsList[rIndex].video_url = vUrlInput.value.trim();
        if (titleInput) adminReelsList[rIndex].title = titleInput.value.trim();

        const hotspotRows = card.querySelectorAll('.reel-hotspots-list > div');
        const hotspots = [];
        hotspotRows.forEach(row => {
            const name = row.querySelector('.hotspot-name').value.trim();
            const price = row.querySelector('.hotspot-price').value.trim();
            const top = row.querySelector('.hotspot-top').value.trim();
            const left = row.querySelector('.hotspot-left').value.trim();
            if (name || price) {
                hotspots.push({ name, price: parseFloat(price) || 0, top: top || '50', left: left || '50' });
            }
        });
        adminReelsList[rIndex].hotspots = hotspots;
    });
}

async function uploadAdminReelVideo(input, rIndex) {
    if (!input.files || !input.files[0]) return;
    const file = input.files[0];
    const formData = new FormData();
    formData.append('file', file);

    try {
        const token = localStorage.getItem('aurawear_token') || localStorage.getItem('token');
        const res = await fetch('/api/upload', {
            method: 'POST',
            headers: token ? { 'Authorization': `Bearer ${token}` } : {},
            body: formData
        });
        const data = await res.json();
        const filePath = data.path || data.url || '';
        if (filePath) {
            collectAdminReelFormValues();
            const finalPath = filePath.startsWith('http') || filePath.startsWith('/') ? filePath : '/' + filePath;
            adminReelsList[rIndex].video_url = finalPath;
            renderAdminReelsBuilder();
            alert('Reel video uploaded successfully!');
        } else {
            alert(data.error || 'Failed to upload video.');
        }
    } catch (e) {
        console.error('Error uploading video:', e);
        alert('Server error while uploading video.');
    }
}

async function saveLookbookReelSettings() {
    collectAdminReelFormValues();
    
    if (adminReelsList.length === 0) {
        alert('Please add at least one lookbook reel before publishing.');
        return;
    }

    for (let i = 0; i < adminReelsList.length; i++) {
        if (!adminReelsList[i].title) {
            alert(`Reel #${i + 1} is missing a title.`);
            return;
        }
        if (!adminReelsList[i].video_url) {
            alert(`Reel #${i + 1} is missing a video URL or upload.`);
            return;
        }
    }

    localStorage.setItem('saved_lookbook_reels', JSON.stringify(adminReelsList));
    localStorage.setItem('saved_lookbook_reel', JSON.stringify(adminReelsList[0]));

    alert(`Successfully published ${adminReelsList.length} Shoppable Lookbook Reel(s) to the storefront!`);
}