use crate::bookmarks::Bookmark;

/// Returns HTML for the bookmarks management page.
pub fn bookmarks_html(bookmarks: &[Bookmark]) -> String {
    let bookmarks_json = serde_json::to_string(bookmarks).unwrap_or_else(|_| "[]".to_string());
    format!(
        r##"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
  @import url('https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600&display=swap');

  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
    background: #202124;
    color: #e8eaed;
    min-height: 100vh;
    display: flex;
    justify-content: center;
    padding: 48px 24px;
  }}

  .container {{ width: 100%; max-width: 560px; }}

  .header {{ margin-bottom: 32px; }}
  .header h1 {{ font-size: 28px; font-weight: 600; letter-spacing: -0.5px; margin-bottom: 6px; }}
  .header p {{ font-size: 13px; color: #9aa0a6; }}

  .card {{
    background: #292b2e;
    border: 1px solid #3c3c3c;
    border-radius: 12px;
    overflow: hidden;
  }}

  .empty {{
    padding: 40px 20px;
    text-align: center;
    color: #9aa0a6;
    font-size: 13px;
  }}

  .bookmark-row {{
    display: flex;
    align-items: center;
    padding: 12px 20px;
    gap: 12px;
    border-bottom: 1px solid #35363a;
    transition: background 0.15s;
  }}
  .bookmark-row:last-child {{ border-bottom: none; }}
  .bookmark-row:hover {{ background: #2f3134; }}

  .bm-icon {{
    width: 32px;
    height: 32px;
    background: #35363a;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    color: #8ab4f8;
    font-size: 14px;
  }}

  .bm-info {{
    flex: 1;
    min-width: 0;
  }}
  .bm-name {{
    font-size: 13px;
    font-weight: 500;
    color: #e8eaed;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }}
  .bm-url {{
    font-size: 11px;
    color: #9aa0a6;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin-top: 2px;
  }}

  .bm-actions {{
    display: flex;
    gap: 4px;
    flex-shrink: 0;
    opacity: 0;
    transition: opacity 0.15s;
  }}
  .bookmark-row:hover .bm-actions {{ opacity: 1; }}

  .action-btn {{
    width: 30px;
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    color: #9aa0a6;
    background: none;
    border: none;
    transition: background 0.1s;
  }}
  .action-btn:hover {{ background: #3c3c3c; color: #e8eaed; }}
  .action-btn.delete:hover {{ color: #f28b82; }}

  /* Edit modal */
  .modal-overlay {{
    display: none;
    position: fixed;
    top: 0; left: 0; right: 0; bottom: 0;
    background: rgba(0,0,0,0.5);
    justify-content: center;
    align-items: center;
    z-index: 100;
  }}
  .modal-overlay.show {{ display: flex; }}
  .modal {{
    background: #292b2e;
    border: 1px solid #3c3c3c;
    border-radius: 12px;
    padding: 24px;
    width: 400px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.5);
  }}
  .modal h2 {{ font-size: 16px; font-weight: 600; margin-bottom: 16px; }}
  .modal label {{
    display: block;
    font-size: 12px;
    color: #9aa0a6;
    margin-bottom: 4px;
    margin-top: 12px;
  }}
  .modal input {{
    width: 100%;
    height: 34px;
    background: #202124;
    border: 1px solid #3c3c3c;
    border-radius: 8px;
    padding: 0 12px;
    color: #e8eaed;
    font-size: 13px;
    font-family: 'Inter', sans-serif;
    outline: none;
  }}
  .modal input:focus {{ border-color: #8ab4f8; }}
  .modal-actions {{
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 20px;
  }}
  .btn {{
    height: 34px;
    padding: 0 20px;
    border-radius: 8px;
    font-size: 13px;
    font-weight: 500;
    font-family: 'Inter', sans-serif;
    cursor: pointer;
    border: none;
    transition: all 0.15s;
  }}
  .btn-secondary {{
    background: #35363a;
    color: #e8eaed;
  }}
  .btn-secondary:hover {{ background: #4a4b4f; }}
  .btn-primary {{
    background: #8ab4f8;
    color: #202124;
  }}
  .btn-primary:hover {{ background: #aecbfa; }}

  .toast {{
    position: fixed;
    bottom: 24px;
    left: 50%;
    transform: translateX(-50%) translateY(80px);
    background: #35363a;
    border: 1px solid #4a4b4f;
    border-radius: 10px;
    padding: 10px 20px;
    font-size: 13px;
    color: #e8eaed;
    display: flex;
    align-items: center;
    gap: 8px;
    box-shadow: 0 8px 24px rgba(0,0,0,0.4);
    opacity: 0;
    transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
    pointer-events: none;
  }}
  .toast.show {{ opacity: 1; transform: translateX(-50%) translateY(0); }}
  .toast svg {{ width: 16px; height: 16px; color: #81c995; flex-shrink: 0; }}
</style>
</head>
<body>

<div class="container">
  <div class="header">
    <h1>Bookmarks</h1>
    <p>Manage your saved pages</p>
  </div>
  <div class="card" id="bookmarks-list"></div>
</div>

<div class="modal-overlay" id="edit-modal">
  <div class="modal">
    <h2>Edit Bookmark</h2>
    <label>Name</label>
    <input type="text" id="edit-name" spellcheck="false">
    <label>URL</label>
    <input type="text" id="edit-url" spellcheck="false">
    <div class="modal-actions">
      <button class="btn btn-secondary" onclick="closeModal()">Cancel</button>
      <button class="btn btn-primary" onclick="saveEdit()">Save</button>
    </div>
  </div>
</div>

<div class="toast" id="toast">
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M20 6L9 17l-5-5"/>
  </svg>
  <span id="toast-msg">Done</span>
</div>

<script>
  let bookmarks = {bookmarks_json};
  let editingUrl = null;

  function send(msg) {{
    window.ipc.postMessage(JSON.stringify(msg));
  }}

  function render() {{
    const list = document.getElementById('bookmarks-list');
    if (bookmarks.length === 0) {{
      list.innerHTML = '<div class="empty">No bookmarks yet. Click the star in the address bar to add one.</div>';
      return;
    }}
    list.innerHTML = bookmarks.map((bm, i) => `
      <div class="bookmark-row">
        <div class="bm-icon">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/></svg>
        </div>
        <div class="bm-info">
          <div class="bm-name">${{esc(bm.name)}}</div>
          <div class="bm-url">${{esc(bm.url)}}</div>
        </div>
        <div class="bm-actions">
          <button class="action-btn" onclick="openEdit(${{i}})" title="Edit">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
          </button>
          <button class="action-btn delete" onclick="remove(${{i}})" title="Delete">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
          </button>
        </div>
      </div>
    `).join('');
  }}

  function esc(s) {{
    const d = document.createElement('div');
    d.textContent = s;
    return d.innerHTML;
  }}

  function openEdit(i) {{
    editingUrl = bookmarks[i].url;
    document.getElementById('edit-name').value = bookmarks[i].name;
    document.getElementById('edit-url').value = bookmarks[i].url;
    document.getElementById('edit-modal').classList.add('show');
    document.getElementById('edit-name').focus();
  }}

  function closeModal() {{
    document.getElementById('edit-modal').classList.remove('show');
    editingUrl = null;
  }}

  function saveEdit() {{
    const newName = document.getElementById('edit-name').value;
    const newUrl = document.getElementById('edit-url').value;
    // Remove old, add new
    send({{type:'RemoveBookmark', url: editingUrl}});
    send({{type:'AddBookmark', name: newName, url: newUrl}});
    // Update local state
    bookmarks = bookmarks.map(b => b.url === editingUrl ? {{name: newName, url: newUrl}} : b);
    closeModal();
    render();
    showToast('Bookmark updated');
  }}

  function remove(i) {{
    const url = bookmarks[i].url;
    send({{type:'RemoveBookmark', url: url}});
    bookmarks = bookmarks.filter(b => b.url !== url);
    render();
    showToast('Bookmark removed');
  }}

  function showToast(msg) {{
    const t = document.getElementById('toast');
    document.getElementById('toast-msg').textContent = msg;
    t.classList.add('show');
    setTimeout(() => t.classList.remove('show'), 2000);
  }}

  document.getElementById('edit-modal').addEventListener('click', (e) => {{
    if (e.target.id === 'edit-modal') closeModal();
  }});

  render();
</script>

</body>
</html>"##
    )
}
