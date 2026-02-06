import React from 'react';
import { createRoot } from 'react-dom/client';
import { ConfigProvider } from 'antd';
import zhCN from 'antd/locale/zh_CN';

// 导入 Tauri API 适配层（自动初始化 window.electronAPI 兼容）
import './api/tauriAPI';

const App = React.lazy(() => import('./App'));


// 渲染应用的入口点
const container = document.getElementById('root');
const root = createRoot(container!);

root.render(
  <React.StrictMode>
    <ConfigProvider locale={zhCN}>
      <React.Suspense fallback={(
        <div style={{ height: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#666' }}>
          正在加载应用...
        </div>
      )}
      >
        <App />
      </React.Suspense>
    </ConfigProvider>
  </React.StrictMode>
);
