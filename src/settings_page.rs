/// Returns HTML for a polished settings page, pre-filled with current settings.
pub fn settings_html(default_url: &str) -> String {
    let escaped = default_url.replace('"', "&quot;").replace('<', "&lt;");
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

  .settings-container {{
    width: 100%;
    max-width: 560px;
  }}

  /* Header */
  .header {{
    margin-bottom: 32px;
  }}
  .header h1 {{
    font-size: 28px;
    font-weight: 600;
    letter-spacing: -0.5px;
    color: #e8eaed;
    margin-bottom: 6px;
  }}
  .header p {{
    font-size: 13px;
    color: #9aa0a6;
    font-weight: 400;
  }}

  /* Settings card */
  .settings-card {{
    background: #292b2e;
    border: 1px solid #3c3c3c;
    border-radius: 12px;
    overflow: hidden;
  }}

  .setting-row {{
    display: flex;
    align-items: center;
    padding: 16px 20px;
    gap: 16px;
    transition: background 0.15s;
  }}
  .setting-row:hover {{
    background: #2f3134;
  }}

  .setting-icon {{
    width: 36px;
    height: 36px;
    background: #35363a;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }}
  .setting-icon svg {{
    width: 18px;
    height: 18px;
    color: #8ab4f8;
  }}

  .setting-content {{
    flex: 1;
    min-width: 0;
  }}
  .setting-label {{
    font-size: 13px;
    font-weight: 500;
    color: #e8eaed;
    margin-bottom: 2px;
  }}
  .setting-description {{
    font-size: 11px;
    color: #9aa0a6;
  }}

  .setting-input {{
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
    margin-top: 10px;
    transition: border-color 0.2s, box-shadow 0.2s;
  }}
  .setting-input:focus {{
    border-color: #8ab4f8;
    box-shadow: 0 0 0 3px rgba(138, 180, 248, 0.15);
  }}

  /* Actions */
  .actions {{
    display: flex;
    align-items: center;
    justify-content: flex-end;
    padding: 16px 20px;
    gap: 12px;
    border-top: 1px solid #3c3c3c;
  }}

  .btn {{
    height: 34px;
    padding: 0 20px;
    border-radius: 8px;
    font-size: 13px;
    font-weight: 500;
    font-family: 'Inter', sans-serif;
    cursor: pointer;
    transition: all 0.15s;
    border: none;
  }}

  .btn-primary {{
    background: #8ab4f8;
    color: #202124;
  }}
  .btn-primary:hover {{
    background: #aecbfa;
    transform: translateY(-1px);
    box-shadow: 0 2px 8px rgba(138, 180, 248, 0.3);
  }}
  .btn-primary:active {{
    transform: translateY(0);
  }}

  /* Toast notification */
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
  .toast.show {{
    opacity: 1;
    transform: translateX(-50%) translateY(0);
  }}
  .toast svg {{
    width: 16px;
    height: 16px;
    color: #81c995;
    flex-shrink: 0;
  }}

  /* Version info */
  .version {{
    text-align: center;
    margin-top: 32px;
    font-size: 11px;
    color: #5f6368;
  }}
</style>
</head>
<body>

<div class="settings-container">
  <div class="header">
    <h1>Settings</h1>
    <p>Configure your Light browser preferences</p>
  </div>

  <div class="settings-card">
    <div class="setting-row">
      <div class="setting-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>
          <polyline points="9 22 9 12 15 12 15 22"/>
        </svg>
      </div>
      <div class="setting-content">
        <div class="setting-label">Home page</div>
        <div class="setting-description">Opens when you launch Light or create a new tab</div>
        <input class="setting-input" type="text" id="default-url" value="{escaped}" spellcheck="false"
               onkeydown="if(event.key==='Enter')save()">
      </div>
    </div>
    <div class="actions">
      <button class="btn btn-primary" onclick="save()">Save changes</button>
    </div>
  </div>

  <div class="version">Light Browser v0.1.0</div>
</div>

<div class="toast" id="toast">
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M20 6L9 17l-5-5"/>
  </svg>
  Settings saved
</div>

<script>
  function save() {{
    const url = document.getElementById('default-url').value;
    window.ipc.postMessage(JSON.stringify({{type:'SaveSettings', default_url: url}}));
    const toast = document.getElementById('toast');
    toast.classList.add('show');
    setTimeout(() => toast.classList.remove('show'), 2000);
  }}
</script>

</body>
</html>"##
    )
}
