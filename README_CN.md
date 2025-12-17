# Claude Code Usage Tracker

一个用于追踪 Claude Code 使用统计的桌面应用程序，使用 Tauri、React 和 Rust 构建。

[English Documentation](./README.md)

> **注意**: 数据来源于本地会话文件解析，而非 Anthropic 官方 API。统计数据仅供参考，可能与官方账单数据存在差异。

## 功能特性

- **实时用量追踪**: 监控所有 Claude Code 项目的 token 使用量、费用和消息数量
- **今日统计**: 查看从本地午夜开始的每日使用量，支持时区感知计算
- **项目级分析**: 按单个项目分解使用情况，提供详细统计数据
- **消耗速率监控**: 追踪当前会话的 token 消耗速率和预计每小时费用
- **模型分布**: 通过交互式饼图可视化不同 Claude 模型的使用情况
- **使用趋势**: 通过 30 天趋势图查看历史使用模式
- **紧凑模式**: 最小化悬浮窗口，支持置顶显示，不影响工作流程
- **迷你模式**: 超紧凑单行显示，在紧凑模式下鼠标离开窗口 10 秒后自动激活，仅显示今日费用和 Token
- **会话窗口追踪**: 监控 5 小时滚动会话窗口，显示重置倒计时

## 截图

### 正常模式

#### 总体统计
![总体统计](docs/overall.jpg)

#### 项目视图
![项目视图](docs/projects.jpg)

### 紧凑模式

#### 紧凑总览视图
![紧凑总览](docs/compact-overall.jpg)

#### 紧凑项目视图
![紧凑项目](docs/compact-projects.jpg)

### 迷你模式
![迷你模式](docs/mini.jpg)

## 安装

### 下载预编译版本

从 [Releases](https://github.com/LyndonWangWork/Claude-Code-Usage-Tracker/releases) 页面下载适合您平台的最新版本。

#### 安装版

| 平台                  | 格式                |
| --------------------- | ------------------- |
| Windows               | `.msi`, `.exe`      |
| macOS (Intel)         | `.dmg`              |
| macOS (Apple Silicon) | `.dmg`              |
| Linux                 | `.deb`, `.AppImage` |

#### 绿色版（免安装）

无需安装，下载即可运行：

| 平台    | 格式                          |
| ------- | ----------------------------- |
| Windows | `*_windows_portable.exe`      |
| Linux   | `*_linux_portable.AppImage`   |

### 从源码构建

#### 前置条件

- Node.js 20+
- Rust (stable)
- Tauri CLI

#### 设置

```bash
# 克隆仓库
git clone https://github.com/LyndonWangWork/Claude-Code-Usage-Tracker.git
cd Claude-Code-Usage-Tracker

# 安装前端依赖
cd web
npm install

# 运行开发服务器
cd ..
npm run tauri dev
```

#### 构建

```bash
npm run tauri build
```

## 使用方法

1. **启动应用** - 应用会自动检测您的 Claude Code 数据目录
2. **查看总体统计** - 查看总费用、token 数量、消息数和会话指标
3. **浏览项目** - 切换到项目标签页查看每个项目的详细数据
4. **切换紧凑模式** - 点击紧凑模式按钮切换到紧凑悬浮窗口
5. **迷你模式** - 在紧凑模式下，鼠标离开窗口 10 秒后自动进入迷你模式（单行显示）。双击可还原到紧凑模式
6. **置顶窗口** - 使用图钉按钮保持窗口始终在最前面

## 数据来源

应用程序支持两种数据来源：

### 本地文件（默认）

从 Claude Code 的本地 JSONL 会话文件读取使用数据：
- **默认位置**: `~/.claude/projects/`
- **自定义位置**: 通过 `CLAUDE_CONFIG_DIR` 环境变量设置

### OpenTelemetry 遥测（可选）

当启用 Claude Code 遥测时，应用可以通过 OpenTelemetry 接收实时使用数据：

1. **配置 Claude Code 导出遥测数据**：

```bash
# 启用遥测
export CLAUDE_CODE_ENABLE_TELEMETRY=1

# 配置 OTLP 导出器发送到本地收集器
export OTEL_METRICS_EXPORTER=otlp
export OTEL_LOGS_EXPORTER=otlp
export OTEL_EXPORTER_OTLP_PROTOCOL=http/json
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318
```

2. **启动应用** - 当检测到 `CLAUDE_CODE_ENABLE_TELEMETRY=1` 时，应用会自动：
   - 在端口 4318 启动本地 OTLP HTTP 收集器
   - 接收遥测数据并存储到 SQLite
   - 切换到遥测数据源

3. **数据来源指示器** - UI 会显示当前数据来源：
   - 📁 本地文件 - 从 JSONL 文件读取
   - 📡 遥测数据 - 接收实时遥测数据

**注意**：两种数据源互斥。启用遥测时，应用仅从遥测数据读取。要切换回本地文件，请取消设置 `CLAUDE_CODE_ENABLE_TELEMETRY` 环境变量并重启应用

### 数据源对比

两种数据源提供不同级别的详细信息：

| 功能 | 本地文件 (JSONL) | 遥测数据 |
|------|------------------|----------|
| 总 Token 数 | ✅ | ✅ |
| 总费用 | ✅ | ✅ |
| 今日费用 | ✅ | ✅ |
| 每日使用趋势 | ✅ | ✅ |
| 模型分布 | ✅ | ✅ |
| 消耗速率 (tokens/分钟, 费用/小时) | ✅ | ✅ |
| 会话数 | ✅ | ✅ |
| 消息数 | ✅ | ✅ (估算) |
| **项目级统计** | ✅ | ❌ |
| 会话开始时间 | ✅ | ❌ |
| 重置倒计时 | ✅ | ❌ |
| 缓存 Token (读取/创建) | ✅ | ✅ |

**主要差异**：
- **本地文件**：提供完整数据，包括每个项目的详细统计和会话时间信息
- **遥测数据**：实时数据收集，但缺少项目级别的细分（Claude Code 遥测不包含项目信息）

## 发布

发布通过 GitHub Actions 自动化完成。创建新版本：

1. 更新 `src-tauri/tauri.conf.json` 和 `src-tauri/Cargo.toml` 中的版本号
2. 提交版本更改
3. 创建并推送版本标签：
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
4. GitHub Actions 将自动为所有平台构建
5. 将创建包含构建产物的草稿发布
6. 审核草稿发布后正式发布

## 技术栈

- **前端**: React, TypeScript, Tailwind CSS, Recharts
- **后端**: Rust, Tauri v2
- **构建**: Vite, Cargo

## 致谢

特别感谢 Maciek-roboblog 的 [Claude Code Usage Monitor](https://github.com/Maciek-roboblog/Claude-Code-Usage-Monitor) 项目提供的灵感和参考实现。

