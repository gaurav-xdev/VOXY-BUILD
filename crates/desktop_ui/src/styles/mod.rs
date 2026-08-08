pub const APP_CSS: &str = r#"
:root {
    --bg-primary: #0a0a0f;
    --bg-secondary: #12121a;
    --bg-tertiary: #1a1a2e;
    --bg-elevated: #222240;
    --bg-hover: #2a2a4a;
    --bg-active: #33335a;
    --text-primary: #e8e8f0;
    --text-secondary: #a0a0b8;
    --text-muted: #6a6a80;
    --accent-primary: #6c5ce7;
    --accent-secondary: #a29bfe;
    --accent-glow: rgba(108, 92, 231, 0.3);
    --border: #2a2a3e;
    --border-active: #6c5ce7;
    --success: #00d2d3;
    --warning: #feca57;
    --error: #ff6b6b;
    --info: #54a0ff;
    --orb-idle: #6c5ce7;
    --orb-listening: #00d2d3;
    --orb-speaking: #feca57;
    --orb-thinking: #a29bfe;
    --orb-error: #ff6b6b;
    --sidebar-width: 260px;
    --header-height: 48px;
    --radius-sm: 6px;
    --radius-md: 10px;
    --radius-lg: 16px;
    --radius-xl: 24px;
    --radius-full: 9999px;
    --shadow-sm: 0 1px 3px rgba(0,0,0,0.3);
    --shadow-md: 0 4px 12px rgba(0,0,0,0.4);
    --shadow-lg: 0 8px 24px rgba(0,0,0,0.5);
    --transition-fast: 0.15s ease;
    --transition-normal: 0.25s ease;
    --transition-slow: 0.4s ease;
}

* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

html, body {
    width: 100%;
    height: 100%;
    overflow: hidden;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    background: var(--bg-primary);
    color: var(--text-primary);
    font-size: 14px;
    line-height: 1.5;
    -webkit-font-smoothing: antialiased;
}

#main {
    width: 100%;
    height: 100%;
}

::-webkit-scrollbar {
    width: 6px;
    height: 6px;
}
::-webkit-scrollbar-track {
    background: transparent;
}
::-webkit-scrollbar-thumb {
    background: var(--bg-hover);
    border-radius: var(--radius-full);
}
::-webkit-scrollbar-thumb:hover {
    background: var(--bg-active);
}

.app-layout {
    display: flex;
    width: 100%;
    height: 100%;
}

.sidebar {
    width: var(--sidebar-width);
    height: 100%;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    overflow: hidden;
}

.sidebar-header {
    padding: 16px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 12px;
}

.sidebar-logo {
    width: 32px;
    height: 32px;
    background: var(--accent-primary);
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 700;
    font-size: 14px;
    color: white;
}

.sidebar-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
}

.sidebar-version {
    font-size: 11px;
    color: var(--text-muted);
}

.sidebar-nav {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
}

.nav-section {
    margin-bottom: 8px;
}

.nav-section-label {
    padding: 4px 12px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
}

.nav-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
    color: var(--text-secondary);
    font-size: 13px;
    font-weight: 500;
    border: none;
    background: none;
    width: 100%;
    text-align: left;
}

.nav-item:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
}

.nav-item.active {
    background: var(--accent-glow);
    color: var(--accent-secondary);
}

.nav-item-icon {
    width: 18px;
    height: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 16px;
}

.sidebar-footer {
    padding: 12px 16px;
    border-top: 1px solid var(--border);
}

.user-info {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: background var(--transition-fast);
}

.user-info:hover {
    background: var(--bg-hover);
}

.user-avatar {
    width: 32px;
    height: 32px;
    border-radius: var(--radius-full);
    background: var(--bg-tertiary);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 14px;
    color: var(--text-secondary);
}

.user-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
}

.user-plan {
    font-size: 11px;
    color: var(--text-muted);
}

.main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 0;
}

.content-header {
    height: var(--header-height);
    padding: 0 24px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
}

.content-header h1 {
    font-size: 16px;
    font-weight: 600;
}

.content-body {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
}

.card {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: 20px;
    margin-bottom: 16px;
}

.card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
}

.card-title {
    font-size: 15px;
    font-weight: 600;
}

.btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 8px 16px;
    border-radius: var(--radius-md);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all var(--transition-fast);
    border: 1px solid transparent;
}

.btn-primary {
    background: var(--accent-primary);
    color: white;
}

.btn-primary:hover {
    background: var(--accent-secondary);
}

.btn-secondary {
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    border-color: var(--border);
}

.btn-secondary:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
}

.btn-ghost {
    background: transparent;
    color: var(--text-secondary);
}

.btn-ghost:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
}

.btn-danger {
    background: rgba(255, 107, 107, 0.1);
    color: var(--error);
    border-color: rgba(255, 107, 107, 0.2);
}

.btn-danger:hover {
    background: rgba(255, 107, 107, 0.2);
}

.input {
    width: 100%;
    padding: 10px 14px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    font-size: 13px;
    transition: border-color var(--transition-fast);
    outline: none;
}

.input:focus {
    border-color: var(--accent-primary);
    box-shadow: 0 0 0 3px var(--accent-glow);
}

.input::placeholder {
    color: var(--text-muted);
}

.toggle {
    position: relative;
    width: 44px;
    height: 24px;
    background: var(--bg-tertiary);
    border-radius: var(--radius-full);
    cursor: pointer;
    transition: background var(--transition-fast);
    border: 1px solid var(--border);
}

.toggle.active {
    background: var(--accent-primary);
    border-color: var(--accent-primary);
}

.toggle::after {
    content: '';
    position: absolute;
    top: 2px;
    left: 2px;
    width: 18px;
    height: 18px;
    background: white;
    border-radius: var(--radius-full);
    transition: transform var(--transition-fast);
}

.toggle.active::after {
    transform: translateX(20px);
}

.status-dot {
    width: 8px;
    height: 8px;
    border-radius: var(--radius-full);
    flex-shrink: 0;
}

.status-dot.healthy { background: var(--success); }
.status-dot.degraded { background: var(--warning); }
.status-dot.unhealthy { background: var(--error); }
.status-dot.unknown { background: var(--text-muted); }

.badge {
    display: inline-flex;
    align-items: center;
    padding: 2px 8px;
    border-radius: var(--radius-full);
    font-size: 11px;
    font-weight: 500;
}

.badge-success { background: rgba(0,210,211,0.15); color: var(--success); }
.badge-warning { background: rgba(254,202,87,0.15); color: var(--warning); }
.badge-error { background: rgba(255,107,107,0.15); color: var(--error); }
.badge-info { background: rgba(84,160,255,0.15); color: var(--info); }

.chat-container {
    display: flex;
    flex-direction: column;
    height: 100%;
}

.chat-messages {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 16px;
}

.message {
    display: flex;
    gap: 12px;
    max-width: 80%;
}

.message.user {
    align-self: flex-end;
    flex-direction: row-reverse;
}

.message-avatar {
    width: 32px;
    height: 32px;
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 14px;
    flex-shrink: 0;
}

.message.assistant .message-avatar {
    background: var(--accent-glow);
    color: var(--accent-secondary);
}

.message.user .message-avatar {
    background: var(--bg-tertiary);
    color: var(--text-secondary);
}

.message-bubble {
    padding: 10px 14px;
    border-radius: var(--radius-lg);
    font-size: 13px;
    line-height: 1.6;
}

.message.assistant .message-bubble {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
}

.message.user .message-bubble {
    background: var(--accent-primary);
    color: white;
}

.chat-input-area {
    padding: 16px 24px;
    border-top: 1px solid var(--border);
    display: flex;
    gap: 12px;
    align-items: flex-end;
}

.chat-input-wrapper {
    flex: 1;
    position: relative;
}

.chat-input {
    width: 100%;
    min-height: 44px;
    max-height: 120px;
    padding: 10px 14px;
    padding-right: 48px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    color: var(--text-primary);
    font-size: 13px;
    resize: none;
    outline: none;
    font-family: inherit;
    line-height: 1.5;
}

.chat-input:focus {
    border-color: var(--accent-primary);
}

.send-btn {
    position: absolute;
    right: 8px;
    bottom: 8px;
    width: 32px;
    height: 32px;
    border-radius: var(--radius-md);
    background: var(--accent-primary);
    color: white;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background var(--transition-fast);
}

.send-btn:hover {
    background: var(--accent-secondary);
}

.voice-controls {
    display: flex;
    gap: 8px;
}

.voice-btn {
    width: 44px;
    height: 44px;
    border-radius: var(--radius-full);
    border: 1px solid var(--border);
    background: var(--bg-secondary);
    color: var(--text-secondary);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 18px;
    transition: all var(--transition-fast);
}

.voice-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
}

.voice-btn.active {
    background: var(--accent-primary);
    color: white;
    border-color: var(--accent-primary);
}

.settings-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 16px;
}

.setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 0;
    border-bottom: 1px solid var(--border);
}

.setting-row:last-child {
    border-bottom: none;
}

.setting-label {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
}

.setting-desc {
    font-size: 12px;
    color: var(--text-muted);
    margin-top: 2px;
}

.health-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
    gap: 12px;
}

.health-card {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
}

.health-card-info {
    flex: 1;
    min-width: 0;
}

.health-card-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
}

.health-card-detail {
    font-size: 11px;
    color: var(--text-muted);
    margin-top: 2px;
}

.orb-container {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    position: relative;
}

.orb {
    width: 120px;
    height: 120px;
    border-radius: var(--radius-full);
    background: var(--orb-idle);
    box-shadow: 0 0 40px var(--accent-glow), 0 0 80px rgba(108, 92, 231, 0.15);
    transition: all 0.6s cubic-bezier(0.4, 0, 0.2, 1);
    position: relative;
    cursor: pointer;
}

.orb::before {
    content: '';
    position: absolute;
    inset: -4px;
    border-radius: var(--radius-full);
    background: conic-gradient(from 0deg, transparent, var(--accent-primary), transparent);
    opacity: 0;
    transition: opacity 0.6s ease;
    z-index: -1;
}

.orb.listening {
    background: var(--orb-listening);
    box-shadow: 0 0 50px rgba(0, 210, 211, 0.3), 0 0 100px rgba(0, 210, 211, 0.15);
    animation: pulse-listening 2s ease-in-out infinite;
}

.orb.speaking {
    background: var(--orb-speaking);
    box-shadow: 0 0 50px rgba(254, 202, 87, 0.3), 0 0 100px rgba(254, 202, 87, 0.15);
    animation: pulse-speaking 0.8s ease-in-out infinite;
}

.orb.thinking {
    background: var(--orb-thinking);
    box-shadow: 0 0 50px rgba(162, 155, 254, 0.3), 0 0 100px rgba(162, 155, 254, 0.15);
    animation: spin-glow 3s linear infinite;
}

.orb.error {
    background: var(--orb-error);
    box-shadow: 0 0 50px rgba(255, 107, 107, 0.3);
}

.orb.active::before {
    opacity: 0.6;
    animation: rotate-border 4s linear infinite;
}

@keyframes pulse-listening {
    0%, 100% { transform: scale(1); }
    50% { transform: scale(1.08); }
}

@keyframes pulse-speaking {
    0%, 100% { transform: scale(1); }
    50% { transform: scale(1.04); }
}

@keyframes spin-glow {
    0% { box-shadow: 0 0 50px rgba(162, 155, 254, 0.3) 0deg; }
    100% { box-shadow: 0 0 50px rgba(162, 155, 254, 0.3) 360deg; }
}

@keyframes rotate-border {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
}

.empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 60px 24px;
    text-align: center;
}

.empty-state-icon {
    font-size: 48px;
    margin-bottom: 16px;
    opacity: 0.3;
}

.empty-state-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 8px;
}

.empty-state-desc {
    font-size: 13px;
    color: var(--text-muted);
    max-width: 400px;
}

.download-item {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 14px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    margin-bottom: 8px;
}

.download-icon {
    width: 40px;
    height: 40px;
    border-radius: var(--radius-md);
    background: var(--bg-tertiary);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 18px;
    flex-shrink: 0;
}

.download-info {
    flex: 1;
    min-width: 0;
}

.download-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.download-meta {
    font-size: 11px;
    color: var(--text-muted);
    margin-top: 2px;
}

.progress-bar {
    width: 100%;
    height: 4px;
    background: var(--bg-tertiary);
    border-radius: var(--radius-full);
    margin-top: 6px;
    overflow: hidden;
}

.progress-fill {
    height: 100%;
    background: var(--accent-primary);
    border-radius: var(--radius-full);
    transition: width var(--transition-normal);
}

.notification-item {
    display: flex;
    gap: 12px;
    padding: 14px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    margin-bottom: 8px;
}

.notification-icon {
    width: 32px;
    height: 32px;
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 14px;
    flex-shrink: 0;
}

.notification-icon.info { background: rgba(84,160,255,0.15); color: var(--info); }
.notification-icon.warning { background: rgba(254,202,87,0.15); color: var(--warning); }
.notification-icon.error { background: rgba(255,107,107,0.15); color: var(--error); }

.notification-content {
    flex: 1;
    min-width: 0;
}

.notification-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
}

.notification-message {
    font-size: 12px;
    color: var(--text-secondary);
    margin-top: 2px;
}

.notification-time {
    font-size: 11px;
    color: var(--text-muted);
    margin-top: 4px;
}

.plugin-card {
    padding: 16px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    margin-bottom: 8px;
}

.plugin-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
}

.plugin-name {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
}

.plugin-desc {
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 1.5;
}

.plugin-meta {
    display: flex;
    gap: 12px;
    margin-top: 8px;
    font-size: 11px;
    color: var(--text-muted);
}

.tab-bar {
    display: flex;
    gap: 2px;
    background: var(--bg-tertiary);
    padding: 3px;
    border-radius: var(--radius-md);
    margin-bottom: 16px;
}

.tab {
    flex: 1;
    padding: 8px 16px;
    text-align: center;
    font-size: 13px;
    font-weight: 500;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all var(--transition-fast);
    border: none;
    background: none;
}

.tab:hover {
    color: var(--text-secondary);
}

.tab.active {
    background: var(--bg-secondary);
    color: var(--text-primary);
    box-shadow: var(--shadow-sm);
}

.toast-container {
    position: fixed;
    bottom: 24px;
    right: 24px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    z-index: 1000;
}

.toast {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 16px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    animation: slide-in 0.3s ease;
    min-width: 280px;
}

@keyframes slide-in {
    from { transform: translateX(100%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
}
"#;
