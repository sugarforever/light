/// Returns the HTML string for the browser chrome (tab bar + nav bar).
/// Styled to match mainstream browser conventions (Chrome/Arc-like).
pub fn chrome_html() -> String {
    r##"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    font-size: 12px;
    background: #202124;
    color: #e8eaed;
    user-select: none;
    overflow: hidden;
  }

  /* Tab bar — Chrome-style */
  #tab-bar {
    display: flex;
    align-items: flex-end;
    height: 36px;
    background: #202124;
    padding: 0 8px 0 72px; /* left padding for macOS traffic lights */
    gap: 0;
    -webkit-app-region: drag;
  }
  #tabs-container {
    display: flex;
    align-items: flex-end;
    gap: 0;
    flex: 1;
    overflow: hidden;
    -webkit-app-region: no-drag;
  }
  .tab {
    display: flex;
    align-items: center;
    height: 32px;
    padding: 0 8px 0 12px;
    background: transparent;
    border-radius: 8px 8px 0 0;
    cursor: pointer;
    max-width: 240px;
    min-width: 40px;
    font-size: 12px;
    color: #9aa0a6;
    transition: background 0.1s;
    position: relative;
    -webkit-app-region: no-drag;
  }
  .tab:hover { background: #292b2e; }
  .tab.active {
    background: #35363a;
    color: #e8eaed;
  }
  .tab-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    padding-right: 4px;
  }
  .tab-close {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 10px;
    color: #9aa0a6;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.1s;
    flex-shrink: 0;
  }
  .tab:hover .tab-close, .tab.active .tab-close { opacity: 1; }
  .tab-close:hover { background: #5f6368; color: #fff; }
  #new-tab-btn {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    cursor: pointer;
    font-size: 18px;
    color: #9aa0a6;
    flex-shrink: 0;
    margin-left: 4px;
    margin-bottom: 2px;
    -webkit-app-region: no-drag;
  }
  #new-tab-btn:hover { background: #35363a; }

  /* Nav bar — Chrome-style omnibar */
  #nav-bar {
    display: flex;
    align-items: center;
    height: 34px;
    background: #35363a;
    padding: 0 8px;
    gap: 4px;
  }
  .nav-btn {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    cursor: pointer;
    font-size: 13px;
    color: #9aa0a6;
    background: none;
    border: none;
    flex-shrink: 0;
    transition: background 0.1s;
  }
  .nav-btn:hover { background: #4a4b4f; color: #e8eaed; }
  .nav-btn:active { background: #5f6368; }
  #address-bar {
    flex: 1;
    height: 28px;
    background: #202124;
    border: none;
    border-radius: 14px;
    padding: 0 14px;
    color: #e8eaed;
    font-size: 13px;
    outline: none;
    transition: background 0.2s;
  }
  #address-bar:focus {
    background: #292b2e;
    box-shadow: 0 0 0 2px #8ab4f8;
  }
  #address-bar::selection { background: #3c6db5; }
</style>
</head>
<body>

<div id="tab-bar">
  <div id="tabs-container"></div>
  <div id="new-tab-btn" onclick="send({type:'NewTab'})">+</div>
</div>

<div id="nav-bar">
  <button class="nav-btn" onclick="send({type:'GoBack'})">&#9664;</button>
  <button class="nav-btn" onclick="send({type:'GoForward'})">&#9654;</button>
  <button class="nav-btn" onclick="send({type:'Reload'})">&#8635;</button>
  <input id="address-bar" type="text" spellcheck="false"
         onkeydown="if(event.key==='Enter'){send({type:'Navigate',url:this.value})}">
</div>

<script>
  let tabs = [];
  let activeId = null;
  let dragSrcIdx = null;

  function send(msg) {
    window.ipc.postMessage(JSON.stringify(msg));
  }

  function renderTabs() {
    const container = document.getElementById('tabs-container');
    container.innerHTML = '';
    tabs.forEach((tab, idx) => {
      const el = document.createElement('div');
      el.className = 'tab' + (tab.id === activeId ? ' active' : '');
      el.draggable = true;
      el.innerHTML = '<span class="tab-title">' + escapeHtml(tab.title) + '</span>'
                   + '<span class="tab-close" onclick="event.stopPropagation();send({type:\'CloseTab\',id:' + tab.id + '})">&#215;</span>';
      el.onclick = () => send({type:'SwitchTab', id: tab.id});
      el.ondragstart = (e) => { dragSrcIdx = idx; e.dataTransfer.effectAllowed = 'move'; };
      el.ondragover = (e) => e.preventDefault();
      el.ondrop = (e) => {
        e.preventDefault();
        if (dragSrcIdx !== null && dragSrcIdx !== idx) {
          send({type:'ReorderTab', from: dragSrcIdx, to: idx});
        }
        dragSrcIdx = null;
      };
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
        tabs.push({id: msg.id, title: msg.title, url: msg.url, is_loading: false});
        activeId = msg.id;
        renderTabs();
        document.getElementById('address-bar').value = msg.url;
        break;
      case 'TabClosed':
        tabs = tabs.filter(t => t.id !== msg.id);
        renderTabs();
        break;
      case 'TabUpdated':
        tabs = tabs.map(t => t.id === msg.id ? {...t, title: msg.title, url: msg.url, is_loading: msg.is_loading} : t);
        renderTabs();
        if (msg.id === activeId) {
          document.getElementById('address-bar').value = msg.url;
        }
        break;
      case 'ActiveTabChanged':
        activeId = msg.id;
        renderTabs();
        const at = tabs.find(t => t.id === msg.id);
        if (at) document.getElementById('address-bar').value = at.url;
        break;
      case 'AllTabs':
        tabs = msg.tabs;
        activeId = msg.active_id;
        renderTabs();
        const act = tabs.find(t => t.id === activeId);
        if (act) document.getElementById('address-bar').value = act.url;
        break;
      case 'FocusAddressBar':
        document.getElementById('address-bar').focus();
        document.getElementById('address-bar').select();
        break;
    }
  }
</script>

</body>
</html>"##
        .to_string()
}
