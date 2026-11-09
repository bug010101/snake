# Snake · Rust（贪吃蛇）

用 Rust + [macroquad](https://github.com/not-fl3/macroquad) 写的跨平台贪吃蛇小游戏。
逻辑层（`core`）与渲染层（`app`）分离，核心逻辑有完整单元测试。

- **实时动作感**：蛇持续移动，你实时转向
- **低美术**：纯色块绘制，零图片素材
- **美观**：扁平现代深色风 + 圆角格子
- **无 bug 设计**：输入队列杜绝“快速反向自杀”；移动/吃食/撞墙/撞自身的边界全部单测覆盖
- **多端操控**：键盘方向键 / WASD，手机直接滑动

## 玩法
- 电脑：方向键 或 WASD 转向
- 手机：手指滑动转向
- 游戏结束：空格 或 点击/轻触 重新开始

## 在 GitHub Pages 上试玩（无需本地装 Rust）
1. 把本仓库推到 GitHub 的 `main` 分支
2. 仓库 **Settings → Pages → Source** 选择 **GitHub Actions**
3. 等 Actions 跑完（几分钟），打开 Pages 地址即可在浏览器/手机直接玩

> CI 流程：跑 `core` 逻辑单测 → `clippy` 静态检查 → 编译 wasm → 部署到 Pages。

## 本地开发（可选，需自行安装 Rust）
```bash
cargo test -p snake-core      # 跑逻辑单测
cargo run -p app              # 本地窗口运行（需系统图形库）
cargo build -p app --release --target wasm32-unknown-unknown  # 出 wasm
```

## 工程结构
```
Cargo.toml            workspace
core/                 纯逻辑库（无 macroquad 依赖，可独立单测）
  src/lib.rs          贪吃蛇核心：转向/步进/生成食物/胜负判定 + 单测
app/                  前端（macroquad）：游戏循环、输入、渲染
  src/main.rs
index.html            wasm 加载页（移动端 viewport）
mq_js_bundle.js       miniquad wasm 引导 JS（本地内置，Pages 自包含）
.github/workflows/    GitHub Pages 部署工作流
```

## 二期（待做）
- Android APK 打包 workflow（macroquad 可直接出 APK）
- 如需中文游戏内文字，打包一个 CJK 字体进去（当前用内置字体，仅支持 ASCII）
