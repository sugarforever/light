/// Returns the HTML string for the sidebar chrome.
/// Supports compact (favicon only) and expanded (favicon + title) modes.
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

  /* Toggle button */
  #toggle-btn {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
    color: #6e6e6e;
    background: none;
    border: none;
    margin: 6px auto 2px;
    flex-shrink: 0;
    transition: color 0.15s, background 0.1s;
  }
  #toggle-btn:hover { background: #292b2e; color: #e8eaed; }

  /* Tabs list */
  #tabs-section {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    min-height: 0;
    padding-top: 2px;
  }
  #tabs-section::-webkit-scrollbar { width: 3px; }
  #tabs-section::-webkit-scrollbar-thumb { background: #3c3c3c; border-radius: 2px; }

  /* Expanded tab */
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
    width: 16px;
    height: 16px;
    border-radius: 3px;
    flex-shrink: 0;
    object-fit: contain;
  }
  .tab-favicon-placeholder {
    width: 16px;
    height: 16px;
    border-radius: 3px;
    flex-shrink: 0;
    background: #3c3c3c;
  }
  .tab-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    font-size: 12px;
    margin-left: 8px;
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

  /* Compact mode overrides */
  body.compact .tab {
    justify-content: center;
    padding: 0;
    margin: 2px 4px;
    height: 32px;
    border-radius: 6px;
  }
  body.compact .tab-title { display: none; }
  body.compact .tab-close { display: none; }
  body.compact .tab-favicon,
  body.compact .tab-favicon-placeholder {
    width: 18px;
    height: 18px;
  }

  /* New tab button */
  #new-tab-btn {
    display: flex;
    align-items: center;
    height: 30px;
    padding: 0 12px;
    cursor: pointer;
    color: #9aa0a6;
    font-size: 12px;
    gap: 6px;
    margin: 4px 6px 8px;
    border-radius: 6px;
    flex-shrink: 0;
  }
  #new-tab-btn:hover { background: #292b2e; color: #e8eaed; }
  #new-tab-btn .plus { font-size: 16px; }
  #new-tab-btn .label { }

  body.compact #new-tab-btn {
    justify-content: center;
    padding: 0;
    margin: 4px 4px 8px;
  }
  body.compact #new-tab-btn .label { display: none; }
</style>
</head>
<body class="compact">

<button id="toggle-btn" onclick="send({type:'ToggleSidebar'})" title="Toggle sidebar">
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
    <rect x="3" y="3" width="18" height="18" rx="2"/><line x1="9" y1="3" x2="9" y2="21"/>
  </svg>
</button>

<div id="tabs-section"></div>
<div id="new-tab-btn" onclick="send({type:'NewTab'})">
  <span class="plus">+</span><span class="label">New Tab</span>
</div>

<script>
  let tabs = [];
  let activeId = null;
  let compact = true;

  function send(msg) {
    window.ipc.postMessage(JSON.stringify(msg));
  }

  function setCompact(v) {
    compact = v;
    document.body.className = v ? 'compact' : '';
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
      if (compact) {
        el.title = tab.title;
      }
      container.appendChild(el);
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
    }
  }
</script>

</body>
</html>"##
        .to_string()
}
