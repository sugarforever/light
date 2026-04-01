/// Returns the HTML string for the browser chrome (tab bar + nav bar).
/// This is loaded into a small webview at the top of the window.
/// It communicates with Rust via window.ipc.postMessage(JSON).
pub fn chrome_html() -> String {
    r##"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    font-size: 13px;
    background: #2b2b2b;
    color: #e0e0e0;
    user-select: none;
    overflow: hidden;
  }

  /* Tab bar */
  #tab-bar {
    display: flex;
    align-items: center;
    height: 34px;
    background: #1e1e1e;
    padding: 0 4px;
    gap: 2px;
  }
  #tabs-container {
    display: flex;
    align-items: center;
    gap: 2px;
    flex: 1;
    overflow: hidden;
  }
  .tab {
    display: flex;
    align-items: center;
    height: 28px;
    padding: 0 12px;
    background: #2b2b2b;
    border-radius: 6px 6px 0 0;
    cursor: pointer;
    max-width: 200px;
    min-width: 60px;
    font-size: 12px;
    color: #999;
    transition: background 0.15s;
  }
  .tab:hover { background: #333; }
  .tab.active { background: #3c3c3c; color: #fff; }
  .tab-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }
  .tab-close {
    margin-left: 6px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 11px;
    color: #888;
    cursor: pointer;
  }
  .tab-close:hover { background: #555; color: #fff; }
  #new-tab-btn {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
    cursor: pointer;
    font-size: 16px;
    color: #888;
    flex-shrink: 0;
  }
  #new-tab-btn:hover { background: #333; color: #fff; }

  /* Nav bar */
  #nav-bar {
    display: flex;
    align-items: center;
    height: 36px;
    background: #2b2b2b;
    padding: 0 8px;
    gap: 4px;
    border-top: 1px solid #3c3c3c;
  }
  .nav-btn {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
    color: #888;
    background: none;
    border: none;
  }
  .nav-btn:hover { background: #3c3c3c; color: #fff; }
  #address-bar {
    flex: 1;
    height: 26px;
    background: #1e1e1e;
    border: 1px solid #3c3c3c;
    border-radius: 6px;
    padding: 0 10px;
    color: #e0e0e0;
    font-size: 13px;
    outline: none;
  }
  #address-bar:focus { border-color: #5b9bd5; }
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
