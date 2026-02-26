#!/usr/bin/env node

/**
 * 自动设置 Tauri 配置文件中的 identifier
 * 用法：
 *   node scripts/set-tauri-env.js dev   # 设置为开发环境
 *   node scripts/set-tauri-env.js prod  # 设置为生产环境
 */

const fs = require('fs');
const path = require('path');

const configPath = path.join(__dirname, '../src-tauri/tauri.conf.json');
const mode = process.argv[2] || 'dev';

const identifiers = {
  dev: 'com.yourcompany.myloair.dev',
  prod: 'com.yourcompany.myloair'
};

try {
  const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));
  const targetIdentifier = identifiers[mode];
  
  if (!targetIdentifier) {
    console.error(`❌ 无效的模式: ${mode}. 请使用 'dev' 或 'prod'`);
    process.exit(1);
  }

  if (config.identifier === targetIdentifier) {
    console.log(`✅ Identifier 已经是 ${mode} 模式: ${targetIdentifier}`);
    process.exit(0);
  }

  config.identifier = targetIdentifier;
  fs.writeFileSync(configPath, JSON.stringify(config, null, 2) + '\n', 'utf8');
  
  console.log(`✅ 已切换到 ${mode} 模式: ${targetIdentifier}`);
} catch (error) {
  console.error('❌ 修改配置文件失败:', error.message);
  process.exit(1);
}
