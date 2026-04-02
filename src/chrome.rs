/// Returns the HTML string for the sidebar chrome.
/// Clean layout: tabs only + new tab button.
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

  /* Tabs list */
  #tabs-section {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    min-height: 0;
    padding-top: 8px;
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
    margin: 4px 6px 8px;
    border-radius: 6px;
    flex-shrink: 0;
  }
  #new-tab-btn:hover { background: #292b2e; color: #e8eaed; }
  #new-tab-btn .plus { font-size: 16px; }
</style>
</head>
<body>

<div id="tabs-section"></div>
<div id="new-tab-btn" onclick="send({type:'NewTab'})">
  <span class="plus">+</span> New Tab
</div>

<script>
  let tabs = [];
  let activeId = null;

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
