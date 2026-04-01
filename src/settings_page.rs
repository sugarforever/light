/// Returns HTML for the settings page, pre-filled with current settings.
pub fn settings_html(default_url: &str) -> String {
    let escaped = default_url.replace('"', "&quot;").replace('<', "&lt;");
    format!(
        r##"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    background: #202124;
    color: #e8eaed;
    padding: 40px;
  }}
  h1 {{
    font-size: 24px;
    font-weight: 400;
    margin-bottom: 32px;
    color: #e8eaed;
  }}
  .setting {{
    margin-bottom: 24px;
  }}
  label {{
    display: block;
    font-size: 13px;
    color: #9aa0a6;
    margin-bottom: 8px;
  }}
  input[type="text"] {{
    width: 100%;
    max-width: 500px;
    height: 36px;
    background: #292b2e;
    border: 1px solid #3c3c3c;
    border-radius: 6px;
    padding: 0 12px;
    color: #e8eaed;
    font-size: 14px;
    outline: none;
  }}
  input[type="text"]:focus {{
    border-color: #8ab4f8;
  }}
  button {{
    height: 36px;
    padding: 0 24px;
    background: #8ab4f8;
    color: #202124;
    border: none;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    margin-top: 16px;
  }}
  button:hover {{ background: #aecbfa; }}
  .saved {{
    color: #81c995;
    font-size: 13px;
    margin-left: 12px;
    display: none;
  }}
  .saved.show {{ display: inline; }}
</style>
</head>
<body>
  <h1>Settings</h1>
  <div class="setting">
    <label>Default page (opens when you launch Light)</label>
    <input type="text" id="default-url" value="{escaped}" spellcheck="false">
  </div>
  <button onclick="save()">Save</button>
  <span class="saved" id="saved-msg">Saved!</span>

  <script>
    function save() {{
      const url = document.getElementById('default-url').value;
      window.ipc.postMessage(JSON.stringify({{type:'SaveSettings', default_url: url}}));
      const msg = document.getElementById('saved-msg');
      msg.classList.add('show');
      setTimeout(() => msg.classList.remove('show'), 2000);
    }}
    document.getElementById('default-url').addEventListener('keydown', (e) => {{
      if (e.key === 'Enter') save();
    }});
  </script>
</body>
</html>"##
    )
}
