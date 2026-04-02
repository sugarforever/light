/// Returns the HTML string for the sidebar chrome.
/// Vertical layout: tabs, bookmarks, menu.
pub fn chrome_html() -> String {
    r##"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
  * { margin: 0; padding: 0; box-sizing: border-box; }
  *::selection { background: transparent; }
  body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    font-size: 12px;
    background: #1e1e1e;
    color: #e8eaed;
    user-select: none;
    overflow: hidden;
    height: 100vh;
    display: flex;
    flex-direction: column;
  }

  /* Section label */
  .section-label {
    font-size: 10px;
    color: #6e6e6e;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    padding: 8px 12px 4px;
    flex-shrink: 0;
  }

  /* Tabs list */
  #tabs-section {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    min-height: 0;
    padding-top: 4px;
  }
  #tabs-section::-webkit-scrollbar { width: 4px; }
  #tabs-section::-webkit-scrollbar-thumb { background: #3c3c3c; border-radius: 2px; }
  .tab {
    display: flex;
    align-items: center;
    height: 32px;
    padding: 0 8px 0 12px;
    cursor: pointer;
    color: #9aa0a6;
    transition: background 0.1s;
    border-radius: 6px;
    margin: 1px 6px;
  }
  .tab:hover { background: #292b2e; }
  .tab.active { background: #35363a; color: #e8eaed; }
  .tab-favicon {
    width: 14px;
    height: 14px;
    border-radius: 2px;
    margin-right: 8px;
    flex-shrink: 0;
    object-fit: contain;
  }
  .tab-favicon-placeholder {
    width: 14px;
    height: 14px;
    border-radius: 2px;
    margin-right: 8px;
    flex-shrink: 0;
    background: #3c3c3c;
  }
  .tab-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    font-size: 12px;
  }
  .tab-close {
    width: 18px;
    height: 18px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 11px;
    color: #9aa0a6;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.1s;
    flex-shrink: 0;
  }
  .tab:hover .tab-close, .tab.active .tab-close { opacity: 1; }
  .tab-close:hover { background: #5f6368; color: #fff; }

  #new-tab-btn {
    display: flex;
    align-items: center;
    height: 30px;
    padding: 0 12px;
    cursor: pointer;
    color: #9aa0a6;
    font-size: 12px;
    gap: 6px;
    margin: 2px 6px;
    border-radius: 6px;
    flex-shrink: 0;
  }
  #new-tab-btn:hover { background: #292b2e; color: #e8eaed; }
  #new-tab-btn .plus { font-size: 16px; }

  /* Divider */
  .divider {
    height: 1px;
    background: #2a2a2a;
    margin: 4px 8px;
    flex-shrink: 0;
  }

  /* Bookmarks section */
  #bookmarks-section {
    max-height: 150px;
    overflow-y: auto;
    overflow-x: hidden;
    flex-shrink: 0;
  }
  #bookmarks-section::-webkit-scrollbar { width: 4px; }
  #bookmarks-section::-webkit-scrollbar-thumb { background: #3c3c3c; border-radius: 2px; }
  .bookmark-item {
    display: flex;
    align-items: center;
    height: 28px;
    padding: 0 12px;
    cursor: pointer;
    font-size: 11px;
    color: #bdc1c6;
    border-radius: 6px;
    margin: 1px 6px;
  }
  .bookmark-item:hover { background: #292b2e; color: #e8eaed; }
  .bm-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  /* Bottom bar with menu */
  #bottom-bar {
    display: flex;
    align-items: center;
    padding: 6px 8px;
    gap: 4px;
    flex-shrink: 0;
    border-top: 1px solid #2a2a2a;
  }
  #menu-btn {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
    cursor: pointer;
    font-size: 16px;
    color: #9aa0a6;
    background: none;
    border: none;
  }
  #menu-btn:hover { background: #35363a; }

  /* Menu dropdown */
  #menu-dropdown {
    display: none;
    position: absolute;
    bottom: 40px;
    left: 8px;
    background: #292b2e;
    border: 1px solid #3c3c3c;
    border-radius: 8px;
    padding: 4px 0;
    min-width: 160px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.4);
    z-index: 100;
  }
  #menu-dropdown.visible { display: block; }
  .menu-item {
    display: flex;
    align-items: center;
    height: 30px;
    padding: 0 14px;
    cursor: pointer;
    font-size: 12px;
    color: #e8eaed;
    gap: 8px;
  }
  .menu-item:hover { background: #3c3c3c; }
  .menu-item .icon { color: #9aa0a6; font-size: 13px; width: 16px; text-align: center; }
</style>
</head>
<body>

<div class="section-label">Tabs</div>
<div id="tabs-section"></div>
<div id="new-tab-btn" onclick="send({type:'NewTab'})">
  <span class="plus">+</span> New Tab
</div>

<div class="divider"></div>
<div class="section-label" id="bookmarks-label" style="display:none">Bookmarks</div>
<div id="bookmarks-section"></div>

<div id="bottom-bar">
  <button id="menu-btn" onclick="toggleMenu(event)" title="Menu">&#8942;</button>
</div>

<div id="menu-dropdown">
  <div class="menu-item" onclick="send({type:'Navigate',url:'light://bookmarks'});closeMenu()"><span class="icon">&#9734;</span>Bookmarks</div>
  <div class="menu-item" onclick="send({type:'OpenSettings'});closeMenu()"><span class="icon">&#9881;</span>Settings</div>
</div>

<script>
  let tabs = [];
  let activeId = null;
  let bookmarks = [];
  let bookmarksVisible = true;

  function send(msg) {
    window.ipc.postMessage(JSON.stringify(msg));
  }

  // Keyboard shortcuts
  document.addEventListener('keydown', (e) => {
    if (e.metaKey || e.ctrlKey) {
      switch (e.key) {
        case 't': e.preventDefault(); send({type:'NewTab'}); break;
        case 'w': e.preventDefault(); send({type:'CloseTab', id: activeId}); break;
        case 'l': e.preventDefault(); send({type:'FocusAddressBar'}); break;
        case 'r': e.preventDefault(); send({type:'Reload'}); break;
      }
    }
  });

  function toggleMenu(e) {
    e.stopPropagation();
    document.getElementById('menu-dropdown').classList.toggle('visible');
  }

  function closeMenu() {
    document.getElementById('menu-dropdown').classList.remove('visible');
  }

  document.addEventListener('click', closeMenu);

  function renderTabs() {
    const container = document.getElementById('tabs-section');
    container.innerHTML = '';
    tabs.forEach((tab) => {
      const el = document.createElement('div');
      el.className = 'tab' + (tab.id === activeId ? ' active' : '');
      const favHtml = tab.favicon
        ? '<img class="tab-favicon" src="' + escapeHtml(tab.favicon) + '" onerror="this.className=\'tab-favicon-placeholder\';this.removeAttribute(\'src\')">'
        : '<span class="tab-favicon-placeholder"></span>';
      el.innerHTML = favHtml
                   + '<span class="tab-title">' + escapeHtml(tab.title) + '</span>'
                   + '<span class="tab-close" onclick="event.stopPropagation();send({type:\'CloseTab\',id:' + tab.id + '})">&#215;</span>';
      el.onclick = () => send({type:'SwitchTab', id: tab.id});
      container.appendChild(el);
    });
  }

  function renderBookmarks() {
    const section = document.getElementById('bookmarks-section');
    const label = document.getElementById('bookmarks-label');
    section.innerHTML = '';
    if (bookmarks.length === 0 || !bookmarksVisible) {
      section.style.display = 'none';
      label.style.display = 'none';
      return;
    }
    section.style.display = 'block';
    label.style.display = 'block';
    bookmarks.forEach(bm => {
      const el = document.createElement('div');
      el.className = 'bookmark-item';
      el.innerHTML = '<span class="bm-name">' + escapeHtml(bm.name) + '</span>';
      el.onclick = () => send({type:'Navigate', url: bm.url});
      el.oncontextmenu = (e) => { e.preventDefault(); send({type:'RemoveBookmark', url: bm.url}); };
      section.appendChild(el);
    });
  }

  function escapeHtml(s) {
    const d = document.createElement('div');
    d.textContent = s;
    return d.innerHTML;
  }

  function handleMessage(msg) {
    switch (msg.type) {
      case 'TabCreated':
        tabs.push({id: msg.id, title: msg.title, url: msg.url, favicon: msg.favicon || ''});
        activeId = msg.id;
        renderTabs();
        break;
      case 'TabClosed':
        tabs = tabs.filter(t => t.id !== msg.id);
        renderTabs();
        break;
      case 'TabUpdated':
        tabs = tabs.map(t => t.id === msg.id ? {...t, title: msg.title, url: msg.url, favicon: msg.favicon || t.favicon} : t);
        renderTabs();
        break;
      case 'ActiveTabChanged':
        activeId = msg.id;
        renderTabs();
        break;
      case 'AllTabs':
        tabs = msg.tabs;
        activeId = msg.active_id;
        renderTabs();
        break;
      case 'Bookmarks':
        bookmarks = msg.bookmarks;
        renderBookmarks();
        break;
    }
  }
</script>

</body>
</html>"##
        .to_string()
}
