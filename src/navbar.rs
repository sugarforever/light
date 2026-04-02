/// Returns the HTML for the top navigation bar (address bar, nav buttons, bookmark star, menu).
pub fn navbar_html() -> String {
    r##"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    font-size: 12px;
    background: #292b2e;
    color: #e8eaed;
    user-select: none;
    overflow: hidden;
    height: 100vh;
    display: flex;
    align-items: center;
  }
  #nav-bar {
    display: flex;
    align-items: center;
    width: 100%;
    height: 100%;
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
  .nav-btn:hover { background: #3c3c3c; color: #e8eaed; }
  #address-bar {
    flex: 1;
    height: 26px;
    background: #202124;
    border: none;
    border-radius: 13px;
    padding: 0 12px;
    color: #e8eaed;
    font-size: 13px;
    outline: none;
  }
  #address-bar:focus {
    box-shadow: 0 0 0 2px #8ab4f8;
  }
  #address-bar::selection {
    background: #3c6db5;
    color: #fff;
  }
  #bookmark-btn {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    cursor: pointer;
    font-size: 15px;
    color: #9aa0a6;
    background: none;
    border: none;
    flex-shrink: 0;
  }
  #bookmark-btn:hover { background: #3c3c3c; }
  #bookmark-btn.bookmarked { color: #8ab4f8; }
</style>
</head>
<body>

<div id="nav-bar">
  <button class="nav-btn" onclick="send({type:'GoBack'})">&#9664;</button>
  <button class="nav-btn" onclick="send({type:'GoForward'})">&#9654;</button>
  <button class="nav-btn" onclick="send({type:'Reload'})">&#8635;</button>
  <input id="address-bar" type="text" spellcheck="false"
         onkeydown="if(event.key==='Enter'){send({type:'Navigate',url:this.value})}">
  <button id="bookmark-btn" onclick="toggleBookmark()" title="Bookmark">&#9734;</button>
</div>

<script>
  let tabs = [];
  let activeId = null;
  let bookmarks = [];
  let currentUrl = '';

  function send(msg) {
    window.ipc.postMessage(JSON.stringify(msg));
  }

  // Keyboard shortcuts
  document.addEventListener('keydown', (e) => {
    if (e.metaKey || e.ctrlKey) {
      switch (e.key) {
        case 't': e.preventDefault(); send({type:'NewTab'}); break;
        case 'w': e.preventDefault(); send({type:'CloseTab', id: activeId}); break;
        case 'l': e.preventDefault(); handleMessage({type:'FocusAddressBar'}); break;
        case 'r': e.preventDefault(); send({type:'Reload'}); break;
      }
    }
  });

  function updateBookmarkBtn() {
    const btn = document.getElementById('bookmark-btn');
    const isBookmarked = bookmarks.some(b => b.url === currentUrl);
    btn.innerHTML = isBookmarked ? '&#9733;' : '&#9734;';
    btn.className = isBookmarked ? 'bookmarked' : '';
    btn.id = 'bookmark-btn';
  }

  function toggleBookmark() {
    const isBookmarked = bookmarks.some(b => b.url === currentUrl);
    if (isBookmarked) {
      send({type:'RemoveBookmark', url: currentUrl});
    } else {
      const activeTab = tabs.find(t => t.id === activeId);
      const name = activeTab ? activeTab.title : currentUrl;
      send({type:'AddBookmark', name: name, url: currentUrl});
    }
  }

  function handleMessage(msg) {
    switch (msg.type) {
      case 'TabCreated':
        tabs.push({id: msg.id, title: msg.title, url: msg.url, favicon: msg.favicon || ''});
        activeId = msg.id;
        currentUrl = msg.url;
        document.getElementById('address-bar').value = msg.url;
        updateBookmarkBtn();
        break;
      case 'TabUpdated':
        tabs = tabs.map(t => t.id === msg.id ? {...t, title: msg.title, url: msg.url, favicon: msg.favicon || t.favicon} : t);
        if (msg.id === activeId) {
          currentUrl = msg.url;
          document.getElementById('address-bar').value = msg.url;
          updateBookmarkBtn();
        }
        break;
      case 'ActiveTabChanged':
        activeId = msg.id;
        const at = tabs.find(t => t.id === msg.id);
        if (at) {
          currentUrl = at.url;
          document.getElementById('address-bar').value = at.url;
          updateBookmarkBtn();
        }
        break;
      case 'AllTabs':
        tabs = msg.tabs;
        activeId = msg.active_id;
        const act = tabs.find(t => t.id === activeId);
        if (act) {
          currentUrl = act.url;
          document.getElementById('address-bar').value = act.url;
          updateBookmarkBtn();
        }
        break;
      case 'Bookmarks':
        bookmarks = msg.bookmarks;
        updateBookmarkBtn();
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
