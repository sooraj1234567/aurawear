// WHITE SCREEN FIX - minimal admin.js that never crashes
console.log('Admin JS Fixed Loading');
document.addEventListener("DOMContentLoaded",()=>{
  try{
    const navBtns=document.querySelectorAll('.nav-btn');
    const panes=document.querySelectorAll('.tab-pane');
    navBtns.forEach(btn=>{
      btn.addEventListener('click',(e)=>{
        navBtns.forEach(b=>b.classList.remove('active'));
        panes.forEach(p=>p.classList.remove('active'));
        const id=e.currentTarget.getAttribute('data-target');
        e.currentTarget.classList.add('active');
        const pane=document.getElementById(id);
        if(pane){
          pane.classList.add('active');
          if(id==='sellers') loadSellers();
          if(id==='orders' || id==='fulfillment' || id==='orders-fulfillment') loadOrders();
        }
      });
    });
    loadSellers();
    // Also try to load orders on start if orders pane is visible
    const ordersPane = document.getElementById('orders') || document.getElementById('fulfillment') || document.getElementById('orders-fulfillment');
    if(ordersPane && ordersPane.classList.contains('active')) loadOrders();
    // Auto-load orders for your screenshot page (admin.html Orders & Fulfillment)
    loadOrders();
  }catch(err){ console.error(err); }
});
function extractId(v){ if(!v) return ""; if(typeof v==='string') return v; if(typeof v==='object'){ if(v.$oid) return v.$oid; if(v.id) return extractId(v.id); if(v._id) return extractId(v._id); } return String(v); }
let vendorsData=[];
async function loadSellers(){
  const tbody=document.getElementById('vendor-tbody');
  if(tbody) tbody.innerHTML='<tr><td colspan=7 style="padding:20px;text-align:center;">Loading vendors from localStorage...</td></tr>';
  let raw=[];
  try{
    const users=JSON.parse(localStorage.getItem('registeredUsers')||'[]');
    raw=users.filter(u=> (u.role||'').toLowerCase().includes('vendor') || u.storeName || u.store).map(u=>({ id:u.id||u._id||u.email, store:u.storeName||u.store||u.name||'Store', owner:u.name||'Owner', email:u.email||'', status:u.status||'pending' }));
  }catch{}
  const approved=JSON.parse(localStorage.getItem('marketplace_approved_vendors')||'{}');
  const saved=JSON.parse(localStorage.getItem('saved_vendor_statuses')||'{}');
  vendorsData=raw.map(s=>{
    const sid=String(s.id||'').toLowerCase(); const sem=String(s.email||'').toLowerCase();
    let st=s.status||'pending';
    if(approved[sid]==='active' || approved[sem]==='active' || saved[sid]==='active' || saved[sem]==='active') st='active';
    return {...s, status:st};
  });
  if(tbody){
    if(vendorsData.length===0) tbody.innerHTML='<tr><td colspan=7 style="padding:20px;text-align:center;opacity:0.6;">No vendors in registeredUsers. Register vendor first.</td></tr>';
    else tbody.innerHTML=vendorsData.map(v=>`<tr><td><b>${v.store}</b><br><span style="font-size:10px;opacity:0.6;">${v.email}</span></td><td>${v.owner}</td><td>${v.email}</td><td>-</td><td>-</td><td><span class="status-badge ${v.status}">${v.status.toUpperCase()}</span></td><td><button class="btn-action" onclick="approve('${extractId(v.id)}','${v.email}')">APPROVE</button></td></tr>`).join('');
  }
  const pendEl=document.getElementById('pending-count'); const vendEl=document.getElementById('vendor-count');
  if(pendEl) pendEl.innerText=vendorsData.filter(v=>v.status==='pending').length;
  if(vendEl) vendEl.innerText=vendorsData.filter(v=>v.status==='active').length;
}
function approve(id,email){
  let users=JSON.parse(localStorage.getItem('registeredUsers')||'[]');
  users=users.map(u=>{ if(String(u.id||u._id||'').toLowerCase()===String(id).toLowerCase() || String(u.email||'').toLowerCase()===String(email).toLowerCase()) return {...u,status:'active'}; return u; });
  localStorage.setItem('registeredUsers',JSON.stringify(users));
  let approved=JSON.parse(localStorage.getItem('marketplace_approved_vendors')||'{}'); approved[String(id).toLowerCase()]='active'; approved[String(email).toLowerCase()]='active'; localStorage.setItem('marketplace_approved_vendors',JSON.stringify(approved));
  let saved=JSON.parse(localStorage.getItem('saved_vendor_statuses')||'{}'); saved[String(id).toLowerCase()]='active'; saved[String(email).toLowerCase()]='active'; localStorage.setItem('saved_vendor_statuses',JSON.stringify(saved));
  alert(email+' APPROVED permanently! Now reload, logout/login will still be ACTIVE');
  loadSellers();
}
function adminLogout(){ if(confirm('Logout?')){ localStorage.removeItem('aurawear_token'); localStorage.removeItem('token'); location.href='/index.html'; } }
function openVendorModal(id){ const v=vendorsData.find(x=>extractId(x.id)===extractId(id)); if(v) approve(v.id,v.email); }
function forceCloseModal(id){ const m=document.getElementById(id); if(m) m.classList.remove('open'); }

// ====== FIX: Admin now sees new buyer orders (Your Screenshot Bug) ======
async function fetchAllAdminOrdersCombined(){
  let combined=[];
  // 1. Backend API - where checkout.html POSTs to
  try{
    const res=await fetch('/api/orders');
    if(res.ok){
      const data=await res.json();
      const apiOrders=data.orders|| (Array.isArray(data)?data:[]);
      combined=combined.concat(apiOrders);
    }
  }catch(e){ console.log('API fetch failed, using localStorage only'); }
  // 2. All localStorage keys - checkout.html now saves to marketplace_orders
  const keys=['marketplace_orders','aurawear_orders','orders','checkout_orders','order_history','aurawear_cart_orders'];
  keys.forEach(k=>{
    try{
      const arr=JSON.parse(localStorage.getItem(k)||'[]');
      if(Array.isArray(arr)) combined=combined.concat(arr);
    }catch{}
  });
  // Deduplicate
  const seen=new Set(); let uniq=[];
  combined.forEach(o=>{
    const id=String(o._id||o.id||o.order_id||JSON.stringify(o).slice(0,30));
    if(!seen.has(id)){ seen.add(id); uniq.push(o); }
  });
  uniq.sort((a,b)=> new Date(b.created_at||b.date||0) - new Date(a.created_at||a.date||0));
  return uniq;
}

async function loadOrders(){
  const tbody = document.getElementById('orders-tbody') || document.getElementById('fulfillment-tbody') || document.querySelector('#orders table tbody') || document.querySelector('.orders-table tbody');
  const orderTable = document.getElementById('ordersTable') || document.querySelector('table');
  // Find your screenshot table: ORDER ID, CUSTOMER, ITEMS, TOTAL, DATE, STATUS / PAYMENT, ACTIONS
  const allTbodys = document.querySelectorAll('tbody');
  let targetTbody = null;
  // Use the tbody that is inside Orders & Fulfillment section
  for(let tb of allTbodys){
    if(tb.innerHTML.includes('AW-') || tb.id.includes('order') || tb.parentElement.innerHTML.includes('ORDER ID')){
      targetTbody = tb;
      break;
    }
  }
  if(!targetTbody) targetTbody = document.querySelector('tbody');
  if(targetTbody) targetTbody.innerHTML='<tr><td colspan=7 style="padding:20px;text-align:center;">Loading orders from API + localStorage...</td></tr>';

  const orders = await fetchAllAdminOrdersCombined();
  console.log('ADMIN TOTAL ORDERS:', orders.length);

  if(!orders.length){
    if(targetTbody) targetTbody.innerHTML='<tr><td colspan=7 style="padding:20px;text-align:center;opacity:0.6;">No orders found. Place a test order from checkout.html - it will appear here.</td></tr>';
    return;
  }

  if(targetTbody){
    targetTbody.innerHTML = orders.map(o=>{
      const oid = extractId(o._id||o.id||o.order_id);
      const shortId = oid.substring(Math.max(0, oid.length-6)).toUpperCase();
      const customer = o.customer_name||o.name||o.email||o.customer_email||'Customer';
      const email = o.email||o.customer_email||'';
      const itemsCount = (o.items||[]).length || 1;
      const total = Number(o.total||o.grand_total||0).toFixed(2);
      const date = (o.created_at||o.date||'').substring(0,10) || new Date().toISOString().substring(0,10);
      const status = (o.status||'pending').toUpperCase();
      const payment = (o.payment_status||o.payment_method||'PAID').toUpperCase();
      return `<tr>
        <td style="font-family:monospace;font-size:11px;">#AW-${shortId}</td>
        <td><b style="font-size:12px;">${customer}</b><br><span style="font-size:10px;opacity:0.6;">${email}</span></td>
        <td>${itemsCount} items</td>
        <td><b>$${total}</b></td>
        <td>${date}</td>
        <td><span style="background:#d1fae5;padding:4px 8px;border-radius:4px;font-size:10px;font-weight:bold;">${status}</span><br><span style="background:#d1fae5;padding:2px 6px;border-radius:3px;font-size:9px;margin-top:4px;display:inline-block;">${payment}</span></td>
        <td><button onclick="alert('Order ID: ${oid}\\nCustomer: ${customer}\\nTotal: $${total}\\nStatus: ${status}')" style="background:#111;color:#fff;border:none;padding:6px 12px;font-size:10px;cursor:pointer;">👁 VIEW</button></td>
      </tr>`;
    }).join('');
  }
}

// Make REFRESH ORDERS button work for your screenshot
window.refreshOrders = loadOrders;
window.loadOrders = loadOrders;
window.fetchAllAdminOrdersCombined = fetchAllAdminOrdersCombined;