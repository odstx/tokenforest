# TokenForest

[English](README.md) | [中文](README_CN.md)

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

现代化的全栈应用，用于管理 AI API 密钥和 Token 池。

## 功能特性

- JWT 用户认证
- API 密钥管理（支持 CIDR 访问限制）
- Token 池管理（支持多种 AI 模型）
- 数据统计仪表盘
- Swagger UI API 文档
- 响应式 Web 界面

## 技术栈

| 层级 | 技术 |
|------|------|
| 后端 | Rust, Axum, SQLx, SQLite, Tokio |
| 前端 | SvelteKit, TypeScript, Tailwind CSS, DaisyUI |
| 运行时 | Bun (前端), Tokio (后端) |

## 快速开始

```bash
make dev
```

| 服务 | 地址 |
|------|------|
| 前端 | http://localhost:5173 |
| 后端 API | http://localhost:3000 |
| Swagger UI | http://localhost:3000/swagger-ui |

## API 参考

### 认证
| 方法 | 端点 | 描述 |
|------|------|------|
| POST | `/api/auth/register` | 注册用户 |
| POST | `/api/auth/login` | 登录 |

### API 密钥
| 方法 | 端点 | 描述 |
|------|------|------|
| GET | `/api/api-keys` | 列出密钥 |
| POST | `/api/api-keys` | 创建密钥 |
| DELETE | `/api/api-keys/:id` | 删除密钥 |
| PUT | `/api/api-keys/:id/toggle` | 启用/禁用密钥 |

### Token 池
| 方法 | 端点 | 描述 |
|------|------|------|
| GET | `/api/token-pools` | 列出池 |
| POST | `/api/token-pools` | 创建池 |
| PUT | `/api/token-pools/:id` | 更新池 |
| DELETE | `/api/token-pools/:id` | 删除池 |
| PUT | `/api/token-pools/:id/toggle` | 启用/禁用池 |
| POST | `/api/token-pools/:id/test` | 测试连接 |

## 开发命令

| 命令 | 描述 |
|------|------|
| `make dev` | 启动后端 + 前端 |
| `make build-backend` | 发布构建（后端） |
| `make build-frontend` | 生产构建（前端） |
| `make clean` | 清理构建产物 |

```bash
# 代码检查和测试
cd frontend && bun run check
cd backend && cargo clippy && cargo test
```

## 配置

后端 `.dev.env`:
```
DATABASE_URL=sqlite:./database/tokenforest.db?mode=rwc
JWT_SECRET=your-secret-key
HOST=0.0.0.0
PORT=3000
```

## 许可证

[GPL-3.0](LICENSE)
